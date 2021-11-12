use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{IoSliceMut, Read};
use std::path::Path;

pub(crate) struct FileWithProgress {
    file: File,
    progress: ProgressBar,
}

impl FileWithProgress {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(FileWithProgress {
            file,
            progress: ProgressBar::new(len).with_style(ProgressStyle::default_bar().template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} Processing file...",
            )),
        })
    }
}

impl Read for FileWithProgress {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = self.file.read(buf)?;
        self.progress.inc(res as u64);
        Ok(res)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let res = self.file.read_vectored(bufs)?;
        self.progress.inc(res as u64);
        Ok(res)
    }
}

impl Drop for FileWithProgress {
    fn drop(&mut self) {
        self.progress.finish_and_clear();
    }
}
