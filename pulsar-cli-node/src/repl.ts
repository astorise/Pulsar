import { createInterface } from "node:readline";
import { ClientMessage } from "./protocol.js";

export interface ReplHandle {
  done: Promise<void>;
  close: () => void;
}

export function startRepl(send: (message: ClientMessage) => void): ReplHandle {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: "pulsar> ",
    terminal: process.stdin.isTTY === true,
  });

  rl.prompt();

  const done = new Promise<void>((resolveDone) => {
    rl.on("line", (raw) => {
      const trimmed = raw.trim();
      if (trimmed.length === 0) {
        rl.prompt();
        return;
      }
      const lower = trimmed.toLowerCase();
      if (lower === "exit" || lower === "quit") {
        rl.close();
        return;
      }
      send({ type: "user_message", content: trimmed });
      rl.prompt();
    });
    rl.once("close", () => resolveDone());
  });

  return {
    done,
    close: () => rl.close(),
  };
}

export async function promptYesNo(question: string): Promise<boolean> {
  return new Promise((resolveAnswer) => {
    const rl = createInterface({ input: process.stdin, output: process.stdout });
    rl.question(question, (answer) => {
      rl.close();
      resolveAnswer(/^y(es)?$/i.test(answer.trim()));
    });
  });
}
