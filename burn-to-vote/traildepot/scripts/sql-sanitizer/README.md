## SQL-Sanitizer

The code in this directory is a subset of the module obtained at:

(https://github.com/pariazar/perfect-express-sanitizer
)[https://github.com/pariazar/perfect-express-sanitizer]

Much of the code has been removed because the immediate use is not for a full Express.js application. Rather, only pure JS, no-dependency functionality has been ported in order to support Trailbase's `scripts` implementation, which does not offer a full Node.js runtime. 

The following files have been ported without some minor changes:

*  [index.js](https://github.com/pariazar/perfect-express-sanitizer/blob/7a1f269f38d3190f09143b5f7e565486147044fe/index.js)
*  [data/sql.js](https://github.com/pariazar/perfect-express-sanitizer/blob/7a1f269f38d3190f09143b5f7e565486147044fe/data/sql.js)
*  [modules/sql-injection.js](https://github.com/pariazar/perfect-express-sanitizer/blob/7a1f269f38d3190f09143b5f7e565486147044fe/modules/sql_injection.js)

Per the terms of the MIT License, a copy of the license is inlucded at `./LICENSE` along with the copied/modified software. 