mod console {

    struct console {

        fn load_cartridge() {

        }


        fn play_game() {
            //General idea:
            //Loop that executes a single CPU cycle then sequentially executes three PPU cycles
            //Once a full frame has elapsed draw it to the screen
            //Then wait if needed to maintain 60FPS
        }
    }
}