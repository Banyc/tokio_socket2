# TokioSocket2

To make socket2 work with tokio.

## Usage

```rust
let listen_socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
listen_socket.set_nonblocking(true)?;
listen_socket.set_reuse_address(true)?;
listen_socket.bind(&SocketAddr::from(([127, 0, 0, 1], 0)).into())?;
listen_socket.listen(1)?;

let listener = TokioSocket2::new(listen_socket)?;

let listen_addr = listener.get_ref().local_addr()?.as_socket().unwrap();

let mut client = TcpStream::connect(listen_addr).await?;

let (server_socket, _) = listener.read(|socket| socket.accept()).await?;
let server = TokioSocket2::new(server_socket)?;

client.write_all(b"ping").await?;

let mut buf = [0; 4];
let mut pos = 0;

while pos < 4 {
    let n = server.read(|socket| {
        let buf = unsafe {
            mem::transmute::<&mut [u8], &mut [MaybeUninit<u8>]>(&mut buf[pos..])
        };
        socket.recv(buf)
    }).await?;

    pos += n;
}

assert_eq!(&buf[..4], b"ping");

server.write(|socket| socket.send(b"pong")).await?;

client.read_exact(&mut buf[..4]).await?;

assert_eq!(&buf[..4], b"pong");
```
