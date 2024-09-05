const _log = console.log;
const _info = console.info;
const _warn = console.warn;
const _error = console.error;
export default function console_override(id:string) {
    const target = document.getElementById(id) as HTMLInputElement|null;
    if(!target) {
      console.error(`[Console Override] id="${id}" is not found`);
      return
    };
    // console.log = (...args) => {
    //     target.value += `Log  : ${args}\n`;
    //     _log.apply(console, args);
    // };
    // console.info = (...args) => {
    //     target.value += `Info : ${args}\n`;
    //     _info.apply(console, args);
    // };
    // console.warn = (...args) => {
    //     target.value += `Warn : ${args}\n`;
    //     _warn.apply(console, args);
    // };
    // console.error = (...args) => {
    //     target.value += `Error: ${args}\n`;
    //     _error.apply(console, args);
    // };
}
