## qR

A distributed queue service, like beanstalkd

Consists of:

- Go API for public interface with the queue
- Rust queue engine
- Redis state + metadata store
- Cold postgres payload storage
