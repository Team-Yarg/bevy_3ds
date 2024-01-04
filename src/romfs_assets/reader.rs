use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    pin::{pin, Pin},
};

use bevy::asset::io::{file::FileAssetReader, AssetReader, AssetReaderError};
use futures::AsyncRead;

/// Reads assets from the embedded romfs
pub struct RomfsAssetReader;

fn process_path(path: &Path) -> PathBuf {
    assert!(
        !path.starts_with("romfs:/"),
        "path already starts with romfs, this is invalid"
    );
    PathBuf::from("romfs:/").join(path)
}

struct FileReader(File);
impl AsyncRead for FileReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::task::Poll::Ready(self.get_mut().0.read(buf))
    }
}

fn make_asset_reader<'a>(
    p: &Path,
) -> bevy::utils::BoxedFuture<
    'a,
    Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError>,
> {
    let p = process_path(p);
    Box::pin(async move {
        File::open(&p)
            .map(|f| Box::new(FileReader(f)) as _)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AssetReaderError::NotFound(p)
                } else {
                    AssetReaderError::Io(e)
                }
            })
    })
}

impl AssetReader for RomfsAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError>,
    > {
        make_asset_reader(path)
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::Reader<'a>>, bevy::asset::io::AssetReaderError>,
    > {
        make_asset_reader(&get_meta_path(path))
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<
        'a,
        Result<Box<bevy::asset::io::PathStream>, bevy::asset::io::AssetReaderError>,
    > {
        let path = process_path(path);
        todo!()
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> bevy::utils::BoxedFuture<'a, Result<bool, bevy::asset::io::AssetReaderError>> {
        todo!()
    }
}

fn get_meta_path(path: &Path) -> PathBuf {
    let mut meta_path = path.to_path_buf();
    let mut extension = path
        .extension()
        .expect("asset paths must have extensions")
        .to_os_string();
    extension.push(".meta");
    meta_path.set_extension(extension);
    meta_path
}
