# PiHole Switch

Simple windows desktop controls for pihole with the PiHole APi

## Setup

1. Create a `.env` in the root of the project and fill it with the following

```
PI_HOLE_ADDR=http://192.168.0.102
PI_HOLE_KEY=your_api_key
```

2. Run `cargo build --release` to compile the project into an executable
   - This executable can be found in `target/release`

3. (Optional) Place a shortcut to the executable in your startup folder
   - This can be done by pressing `WIN + R` and typing `shell:startup`
   - create a shortcut to the executable and place it in the startup folder

4. (Optional) Place the app in your Start Menu
   - Find the folder `C:\Users\<YOUR_USERNAME>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs`
   - Create a shortcut to the executable and place it in the Programs folder