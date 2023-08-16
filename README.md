[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)

# ngx_rust_howto

## Getting Started

### Build

`cargo build`

### Running

A basic configuration file is provided in the `conf` directory. Once you've built the project you'll have a usable object in the `target/debug` directory, move this to your desired location and update the `conf/howto.con` file with the proper path and name.

You can now start an NGINX instance and try the module. For example, if you use an argument of "GET", the module will allow all `GET` requests and deny others.

```
curl localhost:8080
proxy passed to backend

curl -X POST localhost:8080
<html>
<head><title>403 Forbidden</title></head>
<body>
<center><h1>403 Forbidden</h1></center>
<hr><center>nginx/1.23.3</center>
</body>
</html>
```

## Contributing

Please see the [contributing guide](https://github.com/f5yacobucci/ngx-rust-howto/blob/main/CONTRIBUTING.md) for guidelines on how to best contribute to this project.

## License

[Apache License, Version 2.0](https://github.com/f5yacobucci/ngx-rust-howto/blob/main/LICENSE)

&copy; [F5, Inc.](https://www.f5.com/) 2023
