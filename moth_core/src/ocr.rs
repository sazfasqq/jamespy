use leptess::LepTess;
use tokio::sync::{mpsc, oneshot};
use tokio::task;

struct OcrRequest {
    image_data: Vec<u8>,
    response: oneshot::Sender<Result<String, OCRError>>,
}

pub struct OcrEngine {
    sender: mpsc::Sender<OcrRequest>,
    shutdown: Option<oneshot::Sender<()>>,
}

#[derive(Debug)]
pub enum OCRError {
    UTFError(std::str::Utf8Error),
    PixError(leptess::leptonica::PixError),
    WorkerMissing,
}

impl std::fmt::Display for OCRError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OCRError::UTFError(ref e) => write!(f, "{e}"),
            OCRError::PixError(ref e) => write!(f, "{e}"),
            OCRError::WorkerMissing => write!(f, "Worker died, cannot process OCR request."),
        }
    }
}

impl std::error::Error for OCRError {}

impl OcrEngine {
    /// Creates a new OCR engine with an internal worker.
    #[must_use]
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(10);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        task::spawn(async move {
            let mut tess = LepTess::new(None, "eng").expect("Failed to initialize Tesseract");

            tokio::select! {
                () = async {
                    while let Some(OcrRequest { image_data, response }) = rx.recv().await {
                        let ocr_text = match tess.set_image_from_mem(&image_data) {
                            Ok(())=> tess.get_utf8_text().map_err(OCRError::UTFError),
                            Err(e) => Err(OCRError::PixError(e)),
                        };

                        let _ = response.send(ocr_text);
                    }
                } => {}

                _ = shutdown_rx => {
                    println!("OCR Worker shutting down...");
                }
            }
        });

        OcrEngine {
            sender: tx,
            shutdown: Some(shutdown_tx),
        }
    }

    /// Sends an image to be processed asynchronously and awaits the result.
    pub async fn process(&self, image_data: Vec<u8>) -> Result<String, OCRError> {
        let (response_tx, response_rx) = oneshot::channel();

        let request = OcrRequest {
            image_data,
            response: response_tx,
        };

        if self.sender.send(request).await.is_err() {
            return Err(OCRError::WorkerMissing);
        }

        match response_rx.await {
            Ok(result) => result,
            Err(_) => Err(OCRError::WorkerMissing),
        }
    }
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OcrEngine {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}
