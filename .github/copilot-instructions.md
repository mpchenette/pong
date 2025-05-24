This project is a "block breaker" type simulation written entirely in Rust.

When suggesting code, you should only suggest use of Rust's `std` library. I do not want to use any external packages in my implementation.

The goal is to have a server-side simulation of the following:
- A 10x10 grid of blocks in the center of the screen
- The left half of the blocks should start as navy grey, the right half should start as navy blue
- 2 balls will bounce around the grid
- 1 ball should start on the left side and should be white and one ball should start on the right side and be black
- The ball that starts on the left should only be able to hit the navy blue blocks and not the navy grey blocks.
- The ball that starts on the right should only be able to hit the navy grey blocks and not the navy blue blocks.
- When either ball hits a block, that block should be converted to the other color.
- There should be a running tally of the number of blocks of each color. It should only display the numbers.