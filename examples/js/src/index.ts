import { readFileSync, writeFileSync, STDIO } from `javy/fs`

export function test() {
  const textEncoder = new TextEncoder();

  const inputBuffer = readFileSync(STDIO.Stdin);
  const inputText = new TextDecoder().decode(inputBuffer);

  const stdoutContent = `${inputText} -- out`;
  const stderrContent = `${inputText} -- err`;

  writeFileSync(STDIO.Stdout, textEncoder.encode(stdoutContent));
  writeFileSync(STDIO.Stderr, textEncoder.encode(stderrContent));
}
