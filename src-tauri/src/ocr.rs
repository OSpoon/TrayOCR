use std::fmt;

#[derive(Debug)]
pub enum OcrError {
    NoTextRecognized,
    Error(String),
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OcrError::NoTextRecognized => write!(f, "No text recognized"),
            OcrError::Error(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for OcrError {}

#[cfg(target_os = "macos")]
pub fn recognize_file(path: &str) -> Result<String, OcrError> {
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::AnyObject;
    use objc2::AnyThread;
    use objc2_core_foundation::CGRect;
    use objc2_foundation::{NSArray, NSDictionary, NSString, NSURL};
    use objc2_vision::{
        VNDetectedObjectObservation, VNImageOption, VNImageRequestHandler, VNRecognizeTextRequest,
        VNRequest, VNRequestTextRecognitionLevel,
    };

    unsafe {
        autoreleasepool(|pool| {
            let ns_path = NSString::from_str(path);
            let url = NSURL::fileURLWithPath(&ns_path);
            let options: Retained<NSDictionary<VNImageOption, AnyObject>> =
                NSDictionary::dictionary();

            let handler = VNImageRequestHandler::initWithURL_options(
                VNImageRequestHandler::alloc(),
                &url,
                &options,
            );

            let request = VNRecognizeTextRequest::init(VNRecognizeTextRequest::alloc());
            request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);

            let en = NSString::from_str("en-US");
            let zh = NSString::from_str("zh-Hans");
            let langs = vec![en, zh];
            let langs_arr = NSArray::from_retained_slice(&langs);
            request.setRecognitionLanguages(&langs_arr);
            request.setUsesLanguageCorrection(true);
            request.setMinimumTextHeight(0.008);
            request.setAutomaticallyDetectsLanguage(true);

            let vn_request: Retained<VNRequest> = request.clone().into_super().into_super();
            handler
                .performRequests_error(&NSArray::from_retained_slice(&[vn_request]))
                .map_err(|e| OcrError::Error(e.to_string()))?;

            if let Some(results) = request.results() {
                if results.is_empty() {
                    return Err(OcrError::NoTextRecognized);
                }

                let mut collected_text = String::new();
                for result in results {
                    let candidates = result.topCandidates(5);
                    let mut first = None;
                    for candidate in candidates {
                        let conf: f32 = candidate.confidence();
                        if conf >= 0.0 {
                            first = Some(candidate);
                            break;
                        }
                    }
                    let Some(candidate) = first else {
                        continue;
                    };
                    let ns_string = candidate.string();
                    let rust_str = ns_string.to_str(pool);
                    if !rust_str.is_empty() {
                        if !collected_text.is_empty() {
                            let obs: &VNDetectedObjectObservation = &result;
                            let bbox: CGRect = obs.boundingBox();
                            if bbox.origin.y < 0.1 {
                                collected_text.push('\n');
                            } else {
                                collected_text.push(' ');
                            }
                        }
                        collected_text.push_str(rust_str);
                    }
                }

                if collected_text.is_empty() {
                    Err(OcrError::NoTextRecognized)
                } else {
                    Ok(collected_text)
                }
            } else {
                Err(OcrError::NoTextRecognized)
            }
        })
    }
}

#[cfg(target_os = "windows")]
pub fn recognize_file(path: &str) -> Result<String, OcrError> {
    use windows::{
        core::HSTRING, Graphics::Imaging::BitmapDecoder, Media::Ocr::OcrEngine,
        Storage::FileAccessMode, Storage::StorageFile,
    };

    let path = normalize_windows_path(
        std::fs::canonicalize(path)
            .map_err(|e| OcrError::Error(e.to_string()))?
            .to_string_lossy()
            .as_ref(),
    );

    let file = futures::executor::block_on(async {
        StorageFile::GetFileFromPathAsync(&HSTRING::from(&path))
            .map_err(|e| OcrError::Error(e.to_string()))?
            .await
            .map_err(|e| OcrError::Error(e.to_string()))
    })
    .map_err(|e| OcrError::Error(e.to_string()))?;

    let stream = futures::executor::block_on(async {
        file.OpenAsync(FileAccessMode::Read)
            .map_err(|e| OcrError::Error(e.to_string()))?
            .await
            .map_err(|e| OcrError::Error(e.to_string()))
    })
    .map_err(|e| OcrError::Error(e.to_string()))?;

    let bitmap = futures::executor::block_on(async {
        let decoder =
            BitmapDecoder::CreateAsync(&stream).map_err(|e| OcrError::Error(e.to_string()))?;
        let d = decoder.await.map_err(|e| OcrError::Error(e.to_string()))?;
        d.GetSoftwareBitmapAsync()
            .map_err(|e| OcrError::Error(e.to_string()))?
            .await
            .map_err(|e| OcrError::Error(e.to_string()))
    })
    .map_err(|e| OcrError::Error(e.to_string()))?;

    let engine = create_ocr_engine()?;

    let result = futures::executor::block_on(async {
        engine
            .RecognizeAsync(&bitmap)
            .map_err(|e| OcrError::Error(e.to_string()))?
            .await
            .map_err(|e| OcrError::Error(e.to_string()))
    })
    .map_err(|e| OcrError::Error(e.to_string()))?;

    let text = result
        .Text()
        .map_err(|e| OcrError::Error(e.to_string()))?
        .to_string();

    if text.is_empty() {
        Err(OcrError::NoTextRecognized)
    } else {
        Ok(text)
    }
}

#[cfg(target_os = "windows")]
fn create_ocr_engine() -> Result<windows::Media::Ocr::OcrEngine, OcrError> {
    use windows::Media::Ocr::OcrEngine;

    match OcrEngine::TryCreateFromUserProfileLanguages() {
        Ok(engine) => Ok(engine),
        Err(profile_error) => {
            let languages = OcrEngine::AvailableRecognizerLanguages().map_err(|e| {
                OcrError::Error(format!("Failed to read Windows OCR languages: {e}"))
            })?;
            let language_count = languages.Size().map_err(|e| {
                OcrError::Error(format!("Failed to count Windows OCR languages: {e}"))
            })?;
            if language_count == 0 {
                return Err(OcrError::Error(
                    "No Windows OCR languages are installed. Install an OCR-capable Windows language pack.".into(),
                ));
            }

            let language = languages.GetAt(0).map_err(|e| {
                OcrError::Error(format!("Failed to read Windows OCR language: {e}"))
            })?;
            OcrEngine::TryCreateFromLanguage(&language).map_err(|e| {
                OcrError::Error(format!(
                    "Failed to create Windows OCR engine. User profile language error: {profile_error}; fallback language error: {e}",
                ))
            })
        }
    }
}

#[cfg(target_os = "windows")]
fn normalize_windows_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{stripped}")
    } else if let Some(stripped) = path.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path.to_string()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn recognize_file(_path: &str) -> Result<String, OcrError> {
    Err(OcrError::Error(
        "System OCR is not implemented for this platform".into(),
    ))
}
