# unix_ipc_rs


This is still a work in progress but this is a easy to use wrapper around the UnixStream en UnixListener crates from the standard lib;
it makes it so that you can create a server and a client instance of a socket connection,
bough of these instances can send and receive messages.
It includes a blocking recv function as wel as a non blocking one.




```rust

// Message to send should implement Serialize and Deserialize from serde
#[derive(Serialize, Deserialize, Debug)]
struct TestMessage {
    value: String,
}



fn main() -> Result<(), Box<dyn std::error::Error>>{


    let s = spawn(|| {
        // Create a server instance of a socket;
        let mut server = IPCSocket::new_server("/tmp/test_socket.sock").unwrap();

        // Loop to receive messages
        loop {
            // This function will not block and will give a Some with the message when it has received a message, else it will return a None
            if let Some(message) = server.recv::<TestMessage>()? {

                dbg!(&message);

                if message.value == "Hello from client" {
                    
                    // send a message to the client;
                    server.send(TestMessage {
                        value: "Hello from server".to_string(),
                    });

                    return;
                }
            }
        }
    });

    
    
    let c = spawn(|| {
        // Create a client instance
        let mut client = IPCSocket::new_client("/tmp/test_socket.sock").unwrap();

        //  Send a message to the server;  
        let message = client.send(TestMessage {
            value: "Hello from client".to_string(),
        });

        // Receive a message this function is blocking will return a message when it is received
        let message = client.recv_blocking::<TestMessage>()?;

        dbg!(message);
    });

    s.join();
    c.join();

    Ok(())

}
```



