use std::path::Path;

pub fn is_supported_format(pb: &Path) -> bool {
    if pb.is_file() {
        if let Some(ext) = pb.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "jpg" | "jpeg" | "png" | "bmp" | "dds" | "exr" | "tga" | "ktx2" | "hdr" => {
                        return true
                    }
                    _ => return false,
                }
            }
        }
    }

    false
}
