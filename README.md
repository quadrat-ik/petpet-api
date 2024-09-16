### info
Elapsed time: 17.216736ms (CatmullRom filter)

Elapsed time: 10.135752ms (Nearest filter)

Avatar used in test: ![avatar](/assets/613651509015740416.png)

## Getting Started

To get started with this api, follow these instructions:

### Prerequisites

1. **Rust**: Ensure you have Rust installed. If not, you can install it from [the official site](https://www.rust-lang.org/).
2. **Cargo**: Comes with Rust, and you'll use it to build and run the server.

### Installation

1. **Clone the Repository**

   ```bash
   git clone https://github.com/messengernew/petpet-api.git
   cd petpet-api
   ```

2. **Set Up Environment**

   Create a `.env` file in the root of the project directory with the following optional configurations:

   ```env
   BIND_IP=0.0.0.0
   BIND_PORT=6969
   ```

   - `BIND_IP`: IP address for the server to bind to (default is `0.0.0.0`).
   - `BIND_PORT`: Port for the server to listen on (default is `6969`).

3. **Build and Run**

   ```bash
   cargo build --release
   cargo run --release
   ```

### Usage

- **Image Request**: Fetch and process images with the following endpoint:

  ```
  GET /{id}?mode={mode}&upd={force_update}&speed={speed}
  ```

  - `id`: The discord user ID to fetch his avatar (must be a number).
  - `mode`: Response format. Can be `gif`, `base64`, or `json`. Defaults to `gif`.
  - `force_update`: If `true`, forces the server to reprocess the image. Defaults to `false`.
  - `speed`: Filter type for the GIF. Can be `no` for slower quality (Catmull-Rom) or anything else for faster quality (Nearest Neighbor).

  **Example:**

  ```bash
  curl "http://localhost:6969/123?mode=json&speed=no"
  ```

### Example Response

- **JSON Response**

  ```json
  {
    "id": "123",
    "image_data": "base64encodeddata..."
  }
  ```

- **Base64 Response**

  ```
  base64encodeddata...
  ```

- **GIF Response**

  The raw GIF data.

### License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.
