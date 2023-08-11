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
        mut f: F,
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
        mut f: F,
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

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;
    use std::net::SocketAddr;
    use std::{io, mem};

    use socket2::{Domain, Protocol, Socket, Type};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use super::TokioSocket2;

    #[tokio::test]
    async fn test() -> io::Result<()> {
        let listen_socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        listen_socket.set_nonblocking(true)?;
        listen_socket.set_reuse_address(true)?;
        listen_socket.bind(&SocketAddr::from(([127, 0, 0, 1], 0)).into())?;
        listen_socket.listen(1)?;

        let listener = TokioSocket2::new(listen_socket)?;

        eprintln!("Listener");

        let listen_addr = listener.get_ref().local_addr()?.as_socket().unwrap();

        let mut client = TcpStream::connect(listen_addr).await?;

        eprintln!("Client");

        let (server_socket, _) = listener.read(|socket| socket.accept()).await?;
        let server = TokioSocket2::new(server_socket)?;

        eprintln!("Server");

        client.write_all(b"ping").await?;

        eprintln!("Client write");

        let mut buf = [0; 4];
        let mut pos = 0;

        while pos < 4 {
            let n = server
                .read(|socket| {
                    let buf = unsafe {
                        mem::transmute::<&mut [u8], &mut [MaybeUninit<u8>]>(&mut buf[pos..])
                    };
                    socket.recv(buf)
                })
                .await?;

            pos += n;
        }

        eprintln!("Server read");

        assert_eq!(&buf[..4], b"ping");

        server.write(|socket| socket.send(b"pong")).await?;

        eprintln!("Server write");

        client.read_exact(&mut buf[..4]).await?;

        eprintln!("Client read");

        assert_eq!(&buf[..4], b"pong");

        Ok(())
    }
}
