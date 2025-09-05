use std::path::Path;
use std::time::SystemTime;
use async_trait::async_trait;
use id3::TagLike;
use crate::services::preview::{
    PreviewProvider, PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent
};

#[cfg(feature = "audio")]
use rodio::Source;

/// Audio preview provider supporting multiple formats using rodio and symphonia
pub struct AudioPreviewProvider {
    #[cfg(feature = "audio")]
    _initialized: bool,
}

impl AudioPreviewProvider {
    pub fn new() -> Result<Self, PreviewError> {
        #[cfg(feature = "audio")]
        {
            Ok(Self {
                _initialized: true,
            })
        }
        
        #[cfg(not(feature = "audio"))]
        {
            Err(PreviewError::AudioError("Audio support not enabled. Enable the 'audio' feature.".to_string()))
        }
    }
}

impl AudioPreviewProvider {

    #[cfg(feature = "audio")]
    fn extract_audio_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        use std::fs::File;
        use std::io::BufReader;
        
        let mut metadata = FileMetadata::new();
        
        // Get file system metadata
        let fs_metadata = std::fs::metadata(file_path)?;
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Try to extract ID3 tags for MP3 files
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if ext.to_lowercase() == "mp3" {
                if let Ok(tag) = id3::Tag::read_from_path(file_path) {
                    metadata.title = tag.title().map(|s| s.to_string());
                    metadata.artist = tag.artist().map(|s| s.to_string());
                    metadata.album = tag.album().map(|s| s.to_string());
                    metadata.year = tag.year().map(|y| y as u32);
                    
                    // Extract additional metadata
                    if let Some(duration) = tag.duration() {
                        metadata.duration = Some(duration as f64);
                    }
                    
                    tracing::debug!("Extracted ID3 tags from {}: title={:?}, artist={:?}", 
                        file_path.display(), metadata.title, metadata.artist);
                }
            }
        }
        
        // Try to get audio metadata using rodio for technical details
        let file = File::open(file_path)
            .map_err(|e| PreviewError::AudioError(format!("Failed to open audio file: {}", e)))?;
        let buf_reader = BufReader::new(file);
        
        // Attempt to decode the audio file to get basic info
        match rodio::Decoder::new(buf_reader) {
            Ok(decoder) => {
                metadata.sample_rate = Some(decoder.sample_rate());
                
                // Get channel count and codec info
                let channels = decoder.channels() as u32;
                metadata.codec = Some(Self::guess_codec_from_extension(file_path));
                
                // Note: rodio doesn't directly provide duration, so we estimate if not already set
                // In a production system, you'd want to use a more specialized audio library
                // like symphonia for detailed metadata extraction
                
                tracing::debug!("Audio file detected - Sample rate: {}, Channels: {}", 
                    metadata.sample_rate.unwrap_or(0), channels);
            }
            Err(e) => {
                tracing::warn!("Failed to decode audio file {}: {}", file_path.display(), e);
                // Fall back to basic metadata
                metadata.codec = Some(Self::guess_codec_from_extension(file_path));
            }
        }
        
        // Try to extract metadata using symphonia for other formats if available
        #[cfg(feature = "metadata")]
        {
            if metadata.duration.is_none() || metadata.title.is_none() {
                if let Ok(symphonia_metadata) = Self::extract_symphonia_metadata(file_path) {
                    // Fill in missing metadata
                    if metadata.duration.is_none() {
                        metadata.duration = symphonia_metadata.duration;
                    }
                    if metadata.title.is_none() {
                        metadata.title = symphonia_metadata.title;
                    }
                    if metadata.artist.is_none() {
                        metadata.artist = symphonia_metadata.artist;
                    }
                    if metadata.album.is_none() {
                        metadata.album = symphonia_metadata.album;
                    }
                    if metadata.year.is_none() {
                        metadata.year = symphonia_metadata.year;
                    }
                    if metadata.sample_rate.is_none() {
                        metadata.sample_rate = symphonia_metadata.sample_rate;
                    }
                    if metadata.bit_rate.is_none() {
                        metadata.bit_rate = symphonia_metadata.bit_rate;
                    }
                }
            }
        }
        
        Ok(metadata)
    }
    
    #[cfg(all(feature = "audio", feature = "metadata"))]
    fn extract_symphonia_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;
        use std::fs::File;
        
        let mut metadata = FileMetadata::new();
        
        // Open the media source
        let file = Box::new(File::open(file_path)
            .map_err(|e| PreviewError::AudioError(format!("Failed to open file: {}", e)))?);
        let media_source_stream = MediaSourceStream::new(file, Default::default());
        
        // Create a probe hint using the file extension
        let mut hint = Hint::new();
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            hint.with_extension(extension);
        }
        
        // Use the default options for metadata and format readers
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        
        // Probe the media source
        let probed = symphonia::default::get_probe().format(&hint, media_source_stream, &fmt_opts, &meta_opts)
            .map_err(|e| PreviewError::AudioError(format!("Failed to probe format: {}", e)))?;
        
        let mut format = probed.format;
        
        // Extract metadata from the format reader
        if let Some(metadata_rev) = format.metadata().current() {
            for tag in metadata_rev.tags() {
                match tag.key.as_str() {
                    "TITLE" => metadata.title = Some(tag.value.to_string()),
                    "ARTIST" => metadata.artist = Some(tag.value.to_string()),
                    "ALBUM" => metadata.album = Some(tag.value.to_string()),
                    "DATE" | "YEAR" => {
                        if let Ok(year) = tag.value.to_string().parse::<u32>() {
                            metadata.year = Some(year);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Extract technical metadata from streams
        if let Some(track) = format.default_track() {
            let params = &track.codec_params;
            
            if let Some(sample_rate) = params.sample_rate {
                metadata.sample_rate = Some(sample_rate);
            }
            
            if let Some(n_frames) = params.n_frames {
                if let Some(sample_rate) = params.sample_rate {
                    metadata.duration = Some(n_frames as f64 / sample_rate as f64);
                }
            }
            
            // Note: bitrate fields are not available in current symphonia version
            // This would need to be calculated from file size and duration
            // metadata.bit_rate = Some(calculated_bitrate);
            
            metadata.codec = Some(format!("{:?}", params.codec));
        }
        
        Ok(metadata)
    }

    #[cfg(feature = "audio")]
    fn guess_codec_from_extension(file_path: &Path) -> String {
        match file_path.extension().and_then(|e| e.to_str()).unwrap_or("") {
            "mp3" => "MP3".to_string(),
            "wav" => "WAV".to_string(),
            "flac" => "FLAC".to_string(),
            "ogg" => "OGG Vorbis".to_string(),
            "aac" => "AAC".to_string(),
            "m4a" => "M4A/AAC".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    #[cfg(feature = "audio")]
    fn generate_waveform_data(
        file_path: &Path, 
        config: &PreviewConfig
    ) -> Result<Vec<f32>, PreviewError> {
        use std::fs::File;
        use std::io::BufReader;
        
        let file = File::open(file_path)
            .map_err(|e| PreviewError::AudioError(format!("Failed to open audio file: {}", e)))?;
        let buf_reader = BufReader::new(file);
        
        let decoder = rodio::Decoder::new(buf_reader)
            .map_err(|e| PreviewError::AudioError(format!("Failed to decode audio: {}", e)))?;
        
        let sample_rate = decoder.sample_rate() as usize;
        let channels = decoder.channels() as usize;
        
        // Collect samples and convert from i16 to f32
        let samples: Vec<f32> = decoder
            .map(|sample| sample as f32 / i16::MAX as f32)
            .collect();
        
        if samples.is_empty() {
            return Ok(vec![0.0; config.audio_waveform_samples]);
        }
        
        // Calculate how many samples to skip for downsampling
        let total_samples = samples.len() / channels; // Mono samples
        let target_samples = config.audio_waveform_samples;
        
        if total_samples <= target_samples {
            // If we have fewer samples than target, just convert to mono
            return Ok(Self::convert_to_mono(&samples, channels));
        }
        
        let skip_factor = total_samples / target_samples;
        let mut waveform = Vec::with_capacity(target_samples);
        
        // Downsample by taking every Nth sample and converting to mono
        for i in 0..target_samples {
            let sample_index = i * skip_factor * channels;
            
            if sample_index < samples.len() {
                // Convert to mono by averaging channels
                let mut sample_sum = 0.0;
                for ch in 0..channels {
                    if sample_index + ch < samples.len() {
                        sample_sum += samples[sample_index + ch];
                    }
                }
                waveform.push(sample_sum / channels as f32);
            } else {
                waveform.push(0.0);
            }
        }
        
        // Normalize waveform to [-1.0, 1.0] range
        Self::normalize_waveform(&mut waveform);
        
        Ok(waveform)
    }

    #[cfg(feature = "audio")]
    fn convert_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
        if channels == 1 {
            return samples.to_vec();
        }
        
        let mono_samples = samples.len() / channels;
        let mut mono = Vec::with_capacity(mono_samples);
        
        for i in 0..mono_samples {
            let mut sum = 0.0;
            for ch in 0..channels {
                let idx = i * channels + ch;
                if idx < samples.len() {
                    sum += samples[idx];
                }
            }
            mono.push(sum / channels as f32);
        }
        
        mono
    }

    #[cfg(feature = "audio")]
    fn normalize_waveform(waveform: &mut [f32]) {
        if waveform.is_empty() {
            return;
        }
        
        // Find the maximum absolute value
        let max_val = waveform.iter()
            .map(|&x| x.abs())
            .fold(0.0f32, |acc, x| acc.max(x));
        
        if max_val > 0.0 {
            // Normalize to [-1.0, 1.0]
            for sample in waveform.iter_mut() {
                *sample /= max_val;
            }
        }
    }

    #[cfg(feature = "audio")]
    fn create_audio_sample(file_path: &Path) -> Result<Option<Vec<u8>>, PreviewError> {
        // For now, we don't generate audio samples
        // This would require more complex audio processing to extract a meaningful preview
        // You could implement this to extract the first few seconds of audio
        tracing::debug!("Audio sample generation not implemented for {}", file_path.display());
        Ok(None)
    }

    #[cfg(not(feature = "audio"))]
    fn extract_audio_metadata_fallback(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        // Fallback implementation without audio processing
        let fs_metadata = std::fs::metadata(file_path)?;
        
        let mut metadata = FileMetadata::new();
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Guess some common audio properties based on file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp3" => {
                    metadata.codec = Some("MP3".to_string());
                    metadata.sample_rate = Some(44100); // Common default
                    metadata.bit_rate = Some(128000); // 128 kbps common
                }
                "wav" => {
                    metadata.codec = Some("WAV".to_string());
                    metadata.sample_rate = Some(44100);
                    metadata.bit_rate = Some(1411000); // 16-bit stereo
                }
                "flac" => {
                    metadata.codec = Some("FLAC".to_string());
                    metadata.sample_rate = Some(44100);
                    metadata.bit_rate = Some(1000000); // Variable
                }
                "ogg" => {
                    metadata.codec = Some("OGG Vorbis".to_string());
                    metadata.sample_rate = Some(44100);
                    metadata.bit_rate = Some(192000);
                }
                _ => {
                    metadata.codec = Some("Unknown".to_string());
                    metadata.sample_rate = Some(44100);
                }
            }
        }
        
        Ok(metadata)
    }

    fn generate_waveform_data_fallback(config: &PreviewConfig) -> Vec<f32> {
        // Generate a simple sine wave as placeholder
        let samples = config.audio_waveform_samples;
        let mut waveform = Vec::with_capacity(samples);
        
        for i in 0..samples {
            let t = i as f32 / samples as f32;
            let freq = 440.0; // A4 note
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
            waveform.push(sample);
        }
        
        waveform
    }

    /// Render waveform data as a thumbnail image
    fn render_waveform_image(waveform_data: &[f32], size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        use image::{RgbImage, DynamicImage, ImageFormat};
        
        let (width, height) = size;
        let mut img = RgbImage::new(width, height);
        
        // Background color (dark)
        let bg_color = image::Rgb([32, 32, 48]);
        let waveform_color = image::Rgb([0, 150, 255]); // Blue waveform
        let center_line_color = image::Rgb([80, 80, 100]); // Gray center line
        
        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg_color;
        }
        
        // Draw center line
        let center_y = height / 2;
        for x in 0..width {
            if (center_y as u32) < height {
                img.put_pixel(x, center_y, center_line_color);
            }
        }
        
        // Draw waveform
        if !waveform_data.is_empty() {
            let samples_per_pixel = waveform_data.len() as f32 / width as f32;
            
            for x in 0..width {
                // Find the sample index for this pixel
                let sample_start = (x as f32 * samples_per_pixel) as usize;
                let sample_end = ((x + 1) as f32 * samples_per_pixel).min(waveform_data.len() as f32) as usize;
                
                // Find min/max amplitude in this range for better visualization
                let mut min_amp = 0.0f32;
                let mut max_amp = 0.0f32;
                
                for i in sample_start..sample_end.min(waveform_data.len()) {
                    let amp = waveform_data[i];
                    min_amp = min_amp.min(amp);
                    max_amp = max_amp.max(amp);
                }
                
                // Convert amplitude to pixel coordinates
                let max_y = (center_y as f32 + max_amp * (height as f32 / 2.0 - 2.0)) as u32;
                let min_y = (center_y as f32 + min_amp * (height as f32 / 2.0 - 2.0)) as u32;
                
                // Draw vertical line for this pixel column
                let start_y = min_y.min(max_y).max(0).min(height - 1);
                let end_y = max_y.max(min_y).max(0).min(height - 1);
                
                for y in start_y..=end_y {
                    if x < width && y < height {
                        img.put_pixel(x, y, waveform_color);
                    }
                }
            }
        }
        
        // Encode as PNG
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        dynamic_img.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| PreviewError::AudioError(format!("Failed to encode waveform image: {}", e)))?;
        
        Ok(buffer)
    }
}

#[async_trait]
impl PreviewProvider for AudioPreviewProvider {
    fn provider_id(&self) -> &'static str {
        "audio"
    }
    
    fn provider_name(&self) -> &'static str {
        "Audio Preview Provider"
    }
    
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_audio()
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma", "aiff", "au"]
    }
    
    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format from extension
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        )
        .filter(|f| f.is_audio())
        .ok_or_else(|| PreviewError::UnsupportedFormat("Not an audio file".to_string()))?;

        #[cfg(feature = "audio")]
        {
            // Extract metadata
            let metadata = Self::extract_audio_metadata(file_path)?;
            
            // Generate waveform data
            let waveform_data = Self::generate_waveform_data(file_path, config)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to generate waveform for {}: {}", file_path.display(), e);
                    Self::generate_waveform_data_fallback(config)
                });
            
            // Generate audio sample (optional)
            let sample_data = Self::create_audio_sample(file_path)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to create audio sample for {}: {}", file_path.display(), e);
                    None
                });
            
            let preview_content = PreviewContent::Audio {
                waveform_data,
                sample_data,
            };
            
            Ok(PreviewData {
                file_path: file_path.to_path_buf(),
                format,
                thumbnail_path: None,
                metadata,
                preview_content,
                generated_at: SystemTime::now(),
            })
        }
        
        #[cfg(not(feature = "audio"))]
        {
            // Fallback implementation without audio processing
            let metadata = Self::extract_audio_metadata_fallback(file_path)?;
            let waveform_data = Self::generate_waveform_data_fallback(config);
            
            let preview_content = PreviewContent::Audio {
                waveform_data,
                sample_data: None,
            };
            
            Ok(PreviewData {
                file_path: file_path.to_path_buf(),
                format,
                thumbnail_path: None,
                metadata,
                preview_content,
                generated_at: SystemTime::now(),
            })
        }
    }
    
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        #[cfg(feature = "audio")]
        {
            Self::extract_audio_metadata(file_path)
        }
        
        #[cfg(not(feature = "audio"))]
        {
            Self::extract_audio_metadata_fallback(file_path)
        }
    }
    
    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // For audio files, generate a simple waveform visualization as thumbnail
        #[cfg(feature = "audio")]
        {
            let config = PreviewConfig::default();
            let waveform = Self::generate_waveform_data(file_path, &config)
                .unwrap_or_else(|_| Self::generate_waveform_data_fallback(&config));
            
            Self::render_waveform_image(&waveform, size)
        }
        
        #[cfg(not(feature = "audio"))]
        {
            let config = PreviewConfig::default();
            let waveform = Self::generate_waveform_data_fallback(&config);
            Self::render_waveform_image(&waveform, size)
        }
    }
    
    fn supports_background_processing(&self) -> bool {
        true // Audio waveform generation benefits from background processing
    }
    
    fn priority(&self) -> u32 {
        250 // Higher priority than generic handlers
    }
}

impl Default for AudioPreviewProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create AudioPreviewProvider: {}", e);
            panic!("Audio support not available");
        })
    }
}

#[async_trait]
impl PreviewHandler for AudioPreviewProvider {
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_audio()
    }

    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format from extension
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        )
        .filter(|f| f.is_audio())
        .ok_or_else(|| PreviewError::UnsupportedFormat("Not an audio file".to_string()))?;

        #[cfg(feature = "audio")]
        {
            // Extract metadata
            let metadata = Self::extract_audio_metadata(file_path)?;
            
            // Generate waveform data
            let waveform_data = Self::generate_waveform_data(file_path, config)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to generate waveform for {}: {}", file_path.display(), e);
                    Self::generate_waveform_data_fallback(config)
                });
            
            // Generate audio sample (optional)
            let sample_data = Self::create_audio_sample(file_path)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to create audio sample for {}: {}", file_path.display(), e);
                    None
                });
            
            let preview_content = PreviewContent::Audio {
                waveform_data,
                sample_data,
            };
            
            Ok(PreviewData {
                file_path: file_path.to_path_buf(),
                format,
                thumbnail_path: None,
                metadata,
                preview_content,
                generated_at: SystemTime::now(),
            })
        }
        
        #[cfg(not(feature = "audio"))]
        {
            // Fallback implementation without audio processing
            let metadata = Self::extract_audio_metadata_fallback(file_path)?;
            let waveform_data = Self::generate_waveform_data_fallback(config);
            
            let preview_content = PreviewContent::Audio {
                waveform_data,
                sample_data: None,
            };
            
            Ok(PreviewData {
                file_path: file_path.to_path_buf(),
                format,
                thumbnail_path: None,
                metadata,
                preview_content,
                generated_at: SystemTime::now(),
            })
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        #[cfg(feature = "audio")]
        {
            Self::extract_audio_metadata(file_path)
        }
        
        #[cfg(not(feature = "audio"))]
        {
            Self::extract_audio_metadata_fallback(file_path)
        }
    }

    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // For audio files, generate a simple waveform visualization as thumbnail
        #[cfg(feature = "audio")]
        {
            let config = PreviewConfig::default();
            let waveform = Self::generate_waveform_data(file_path, &config)
                .unwrap_or_else(|_| Self::generate_waveform_data_fallback(&config));
            
            Self::render_waveform_image(&waveform, size)
        }
        
        #[cfg(not(feature = "audio"))]
        {
            let config = PreviewConfig::default();
            let waveform = Self::generate_waveform_data_fallback(&config);
            Self::render_waveform_image(&waveform, size)
        }
    }
}


// Legacy type alias for backward compatibility
pub type AudioPreviewHandler = AudioPreviewProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_audio_handler_supports_formats() {
        let result = AudioPreviewHandler::new();
        
        #[cfg(feature = "audio")]
        {
            let handler = result.unwrap();
            assert!(handler.supports_format(SupportedFormat::Mp3));
            assert!(handler.supports_format(SupportedFormat::Wav));
            assert!(handler.supports_format(SupportedFormat::Flac));
            assert!(!handler.supports_format(SupportedFormat::Jpeg));
            assert!(!handler.supports_format(SupportedFormat::Mp4));
        }
        
        #[cfg(not(feature = "audio"))]
        {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_audio_metadata_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let audio_path = temp_dir.path().join("test.mp3");
        
        // Create a fake audio file
        fs::write(&audio_path, b"fake audio data").unwrap();
        
        #[cfg(not(feature = "audio"))]
        {
            let metadata = AudioPreviewProvider::extract_audio_metadata_fallback(&audio_path).unwrap();
            assert_eq!(metadata.codec, Some("MP3".to_string()));
            assert_eq!(metadata.sample_rate, Some(44100));
            assert!(metadata.file_size > 0);
        }
    }

    #[test]
    fn test_waveform_fallback_generation() {
        let config = PreviewConfig::default();
        let waveform = AudioPreviewProvider::generate_waveform_data_fallback(&config);
        
        assert_eq!(waveform.len(), config.audio_waveform_samples);
        assert!(waveform.iter().any(|&x| x != 0.0)); // Should contain non-zero values
    }

    #[test]
    fn test_waveform_image_rendering() {
        let waveform_data: Vec<f32> = (0..100)
            .map(|i| (i as f32 / 100.0 * 2.0 * std::f32::consts::PI).sin())
            .collect();
        
        let result = AudioPreviewProvider::render_waveform_image(&waveform_data, (256, 128));
        assert!(result.is_ok());
        
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_audio_file() {
        let temp_dir = TempDir::new().unwrap();
        let text_path = temp_dir.path().join("test.txt");
        fs::write(&text_path, "This is not audio").unwrap();
        
        if let Ok(handler) = AudioPreviewHandler::new() {
            let config = PreviewConfig::default();
            let result = handler.generate_preview(&text_path, &config).await;
            assert!(result.is_err());
            
            match result.unwrap_err() {
                PreviewError::UnsupportedFormat(_) => {}, // Expected
                other => panic!("Expected UnsupportedFormat, got {:?}", other),
            }
        }
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_mono_conversion() {
        let stereo_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]; // 3 stereo samples
        let mono = AudioPreviewProvider::convert_to_mono(&stereo_samples, 2);
        
        assert_eq!(mono.len(), 3);
        assert!((mono[0] - 0.15).abs() < 1e-6); // (0.1 + 0.2) / 2
        assert!((mono[1] - 0.35).abs() < 1e-6); // (0.3 + 0.4) / 2
        assert!((mono[2] - 0.55).abs() < 1e-6); // (0.5 + 0.6) / 2
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_waveform_normalization() {
        let mut waveform = vec![2.0, -4.0, 1.0, -2.0];
        AudioPreviewProvider::normalize_waveform(&mut waveform);
        
        // Should be normalized to max absolute value of 1.0
        assert_eq!(waveform[0], 0.5);   // 2.0 / 4.0
        assert_eq!(waveform[1], -1.0);  // -4.0 / 4.0
        assert_eq!(waveform[2], 0.25);  // 1.0 / 4.0
        assert_eq!(waveform[3], -0.5);  // -2.0 / 4.0
    }
}