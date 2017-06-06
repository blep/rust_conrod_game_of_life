# Game of life in Rust language using Conrod UI.

First small application made as I learn [Rust programming language](https://www.rust-lang.org/en-US/).

Use [Conrod](https://github.com/PistonDevelopers/conrod) for the User Interfance with winit and glium back-end.

# Features

* Turn simulation on/off
* Create or kill cell on the board
* Tweak number of required live neightbor cells to survive or be born simulation run

# Screenshots

![Game of Life with survives with 1,2,5 neighbor cells and born with 3 neighbor cells](screenshots/flower2_125_3.png?raw=true)

![Game of Life classic "Gosper glider gun"](screenshots/gosper_glider_gun_23_3.png?raw=true)

# Building and running

```
cargo run --release
```

# Assets

* Use [Google Noto Fonts](https://www.google.com/get/noto/) (Apache2)

# License

* MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
