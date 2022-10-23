use std::io;

use tokio::io::unix::AsyncFd;

pub struct TokioSocket2 {
    io: AsyncFd<socket2::Socket>,
}

impl TokioSocket2 {
    pub fn new(socket: socket2::Socket) -> io::Result<Self> {
        let io = AsyncFd::new(socket)?;
        Ok(Self { io })
    }

    pub fn get_ref(&self) -> &socket2::Socket {
        self.io.get_ref()
    }

    pub async fn read<F: FnMut(&socket2::Socket) -> io::Result<R>, R>(
        &self,
        mut f: F
    ) -> io::Result<R> {
        loop {
            let mut guard = self.io.readable().await?;

            match guard.try_io(|io| f(io.get_ref())) {
                Ok(result) => {
                    return result;
                }
                Err(_would_block) => {
                    continue;
                }
            }
        }
    }

    pub async fn write<F: FnMut(&socket2::Socket) -> io::Result<R>, R>(
        &self,
        mut f: F
    ) -> io::Result<R> {
        loop {
            let mut guard = self.io.writable().await?;

            match guard.try_io(|io| f(io.get_ref())) {
                Ok(result) => {
                    return result;
                }
                Err(_would_block) => {
                    continue;
                }
            }
        }
    }
}