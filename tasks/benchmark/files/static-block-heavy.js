const sink = [];
const classes = [
  class { static seed = 0; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 1; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 2; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 3; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 4; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 5; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 6; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 7; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 8; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 9; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 10; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 11; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 12; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 13; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 14; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 15; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 16; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 17; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 18; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 19; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 20; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 21; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 22; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
  class { static seed = 23; static { this.seed++; sink.push(this.seed); } static { var local = this.seed; sink.push(local); } },
];

sink.push(classes.length);
