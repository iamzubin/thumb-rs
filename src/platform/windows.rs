/// Windows thumbnail implementation using IShellItemImageFactory.
///
/// Uses the Windows Shell API via the `windows` crate to generate thumbnails
/// that match Explorer.exe — including thumbnails from registered providers
/// like VLC (video frames), Office (document previews), etc.
///
/// # How it works
///
/// 1. `CoInitializeEx(COINIT_APARTMENTTHREADED)` — required for COM
/// 2. `SHCreateItemFromParsingName(path)` → `IShellItemImageFactory`
/// 3. `GetImage(size, flags=0)` — default flags = `SIIGBF_RESIZETOFIT`
///    - This is the SAME call Explorer uses
///    - Invokes registered `IThumbnailProvider` handlers (VLC, Office, etc.)
///    - Falls back to file-type icon if no thumbnail provider exists
///    - Returns an `HBITMAP`
/// 4. Extract pixels: `GetObjectW` → `GetDIBits` → raw BGRA buffer
/// 5. Swap B↔R channels → RGBA
/// 6. Cleanup: `DeleteObject`, `DeleteDC`, `CoUninitialize`
///
/// # Why default flags (0x0) and not SIIGBF_THUMBNAILONLY?
///
/// `SIIGBF_THUMBNAILONLY` forces the shell to only return thumbnails and
/// fails (E_FAIL) if no provider exists. This means .txt, .log, .md files
/// would error instead of returning their file-type icon.
///
/// Default flags (`SIIGBF_RESIZETOFIT` = 0x0) gives you the same behavior
/// as Explorer: real thumbnails when available, icons as fallback.
use crate::error::ThumbsError;
use crate::{Thumbnail, ThumbnailScale};
use std::path::Path;

use windows::core::HSTRING;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, BITMAP,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF};

/// Generate a thumbnail matching Explorer.exe quality.
///
/// Uses `IShellItemImageFactory::GetImage` with default flags (`SIIGBF_RESIZETOFIT`).
/// This invokes the same Shell thumbnail pipeline as Explorer:
///
/// - For video files with VLC installed: returns VLC's extracted frame
/// - For Office docs: returns document preview
/// - For images: returns the image itself (scaled)
/// - For everything else: returns the file-type icon (fallback)
///
/// # Arguments
/// * `file_path` — Path to the source file. Can be any file type.
/// * `scale` — Size multiplier. `1` = 256×256, `2` = 512×512, etc.
///
/// # Returns
/// A `Thumbnail` containing raw RGBA8 pixel data, width, and height.
///
/// # Errors
/// - `ThumbsError::PlatformError` if COM initialization fails or
///   `SHCreateItemFromParsingName` fails (invalid path, access denied).
/// - `ThumbsError::ThumbnailGenerationFailed` if `GetImage` returns an error.
pub fn generate_thumbnail(
    file_path: &Path,
    scale: ThumbnailScale,
) -> Result<Thumbnail, ThumbsError> {
    let path_str = file_path
        .to_str()
        .ok_or_else(|| ThumbsError::PlatformError("Invalid UTF-8 in file path".into()))?;
    let wide_path = HSTRING::from(path_str);

    // Initialize COM (apartment-threaded). S_FALSE means already initialized — ok.
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if hr.is_err() {
        return Err(ThumbsError::PlatformError(format!(
            "CoInitializeEx failed: {hr:?}"
        )));
    }

    // Create shell item from path
    let shell_item: IShellItemImageFactory =
        match unsafe { SHCreateItemFromParsingName(&wide_path, None) } {
            Ok(item) => item,
            Err(e) => {
                unsafe { CoUninitialize() };
                // HRESULT 0x80070002 = ERROR_FILE_NOT_FOUND, 0x80070003 = ERROR_PATH_NOT_FOUND
                let code = e.code().0 as u32;
                if code == 0x80070002 || code == 0x80070003 {
                    return Err(ThumbsError::FileNotFound(file_path.display().to_string()));
                }
                return Err(ThumbsError::PlatformError(format!(
                    "SHCreateItemFromParsingName failed: {e}"
                )));
            }
        };

    // Get thumbnail as HBITMAP
    let hbitmap = match get_hbitmap(&shell_item, scale) {
        Ok(h) => h,
        Err(e) => {
            unsafe { CoUninitialize() };
            return Err(e);
        }
    };

    // Extract RGBA pixels from HBITMAP
    let result = hbitmap_to_rgba(hbitmap);

    // Cleanup: delete the HBITMAP and uninitialize COM
    unsafe {
        let _ = DeleteObject(hbitmap);
        CoUninitialize();
    }

    let (rgba, w, h) = result?;
    Ok(Thumbnail::new(rgba, w, h))
}

/// Call `IShellItemImageFactory::GetImage` to get an HBITMAP.
fn get_hbitmap(
    shell_item: &IShellItemImageFactory,
    scale: ThumbnailScale,
) -> Result<HBITMAP, ThumbsError> {
    let px = scale.px() as i32;
    let dimensions = SIZE { cx: px, cy: px };
    unsafe { shell_item.GetImage(dimensions, SIIGBF(0)) }
        .map_err(|e| ThumbsError::ThumbnailGenerationFailed(format!("GetImage failed: {e}")))
}

/// Extract RGBA8 pixel data from an HBITMAP.
///
/// Uses GDI to read the raw pixel buffer, then converts from the
/// Windows-native BGRA format to platform-independent RGBA.
fn hbitmap_to_rgba(hbitmap: HBITMAP) -> Result<(Vec<u8>, u32, u32), ThumbsError> {
    // Get BITMAP info
    let mut bitmap = BITMAP::default();
    unsafe {
        GetObjectW(
            HGDIOBJ(hbitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );
    }

    let width = bitmap.bmWidth as u32;
    let abs_height = bitmap.bmHeight.unsigned_abs();
    let top_down = bitmap.bmHeight < 0;

    // Create memory DC
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc == HDC::default() {
        return Err(ThumbsError::PlatformError(
            "CreateCompatibleDC failed".into(),
        ));
    }

    // Select bitmap into DC
    let old_obj = unsafe { SelectObject(dc, hbitmap) };

    // Set up BITMAPINFO for 32-bit top-down DIB
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(abs_height as i32), // negative = top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let pixel_count = (width * abs_height) as usize;
    let mut buffer = vec![0u8; pixel_count * 4];

    // Extract raw pixels (BGRA)
    let lines = unsafe {
        GetDIBits(
            dc,
            hbitmap,
            0,
            abs_height,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };

    // Cleanup DC
    unsafe {
        SelectObject(dc, old_obj);
        let _ = DeleteDC(dc);
    }

    if lines == 0 {
        return Err(ThumbsError::ThumbnailGenerationFailed(
            "GetDIBits returned 0 lines".into(),
        ));
    }

    // Convert BGRA → RGBA (swap bytes at [0] and [2] per pixel)
    for pixel in buffer.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    // If original was bottom-up, flip rows
    if !top_down {
        let row_bytes = (width as usize) * 4;
        let mut flipped = vec![0u8; buffer.len()];
        for row in 0..abs_height as usize {
            let src_start = row * row_bytes;
            let dst_start = (abs_height as usize - 1 - row) * row_bytes;
            flipped[dst_start..dst_start + row_bytes]
                .copy_from_slice(&buffer[src_start..src_start + row_bytes]);
        }
        buffer = flipped;
    }

    Ok((buffer, width, abs_height))
}
