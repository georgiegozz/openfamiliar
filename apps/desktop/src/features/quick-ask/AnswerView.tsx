function splitAnswer(
  answer: string,
): Array<{ code: boolean; language: string; value: string }> {
  const chunks: Array<{ code: boolean; language: string; value: string }> = [];
  const matcher = /```([^\n`]*)\n?([\s\S]*?)```/g;
  let cursor = 0;
  for (const match of answer.matchAll(matcher)) {
    if (match.index > cursor) {
      chunks.push({
        code: false,
        language: "",
        value: answer.slice(cursor, match.index),
      });
    }
    chunks.push({
      code: true,
      language: match[1]?.trim() ?? "",
      value: match[2] ?? "",
    });
    cursor = match.index + match[0].length;
  }
  if (cursor < answer.length)
    chunks.push({ code: false, language: "", value: answer.slice(cursor) });
  return chunks;
}

export function AnswerView({ answer }: { answer: string }) {
  return (
    <div className="answer-view" aria-live="polite">
      {splitAnswer(answer).map((chunk, index) =>
        chunk.code ? (
          <div className="code-block" key={index}>
            {chunk.language ? <span>{chunk.language}</span> : null}
            <pre>
              <code>{chunk.value}</code>
            </pre>
          </div>
        ) : (
          <p key={index}>{chunk.value}</p>
        ),
      )}
    </div>
  );
}
