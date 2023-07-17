const fs = require("fs");
const path = require("path");
const shell = require("shelljs");

const projectRoot = path.dirname(require.resolve("./package.json"));
const projectName = require("./package.json").name;
const bin = path.join(projectRoot, "node_modules", ".bin", "share");

test("it's installed", () => {
  expect(fs.existsSync(bin)).toBe(true);
});

test("direct bin can print to stdout and count to 4", () => {
  expect(shell.exec(`${bin} --version`).stdout).toContain("share 0.0.");
});

test("npx can print to stdout and count to 9", () => {
  expect(shell.exec("npx share --help").stdout).toContain(
    "Share anything with teammates across machines via CLI."
  );
});

// test("can receive piped input", () => {
//   expect(
//     shell.echo("hello world").exec(`npx binary-install-example echo`).stdout
//   ).toContain("hello world");
// });