export const doLastTimeFunc = (func: Function, time = 400) => {
  let n: number | null = null
  return async () => {
    clearTimeout(n!)
    n = setTimeout(async () => {
      await func();
    }, time)
  }
}