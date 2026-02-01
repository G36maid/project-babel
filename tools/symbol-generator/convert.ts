import { readdirSync } from "fs";

interface Dictionary {
  [key: string]: string;
}

async function convertDictionaryToSVGs(): Promise<void> {
  // Read dictionary.json
  const dictionaryFile = await Bun.file(
    "tools/symbol-generator/dictionary.json"
  );
  const dictionary: Dictionary = await dictionaryFile.json();

  // Ensure output directory exists
  const outputDir = "frontend/src/assets/symbols";
  await new Promise<void>((resolve, reject) => {
    const proc = Bun.spawn(["mkdir", "-p", outputDir]);
    proc.exited.then(() => resolve()).catch(reject);
  });

  let generatedCount = 0;

  // Process each word
  for (const [word, content] of Object.entries(dictionary)) {
    // Skip censored entry
    if (word === "***") {
      continue;
    }

    // Create SVG with wrapper
    const svgContent = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="none">
${content}
</svg>`;

    // Write to file
    const outputPath = `${outputDir}/${word}.svg`;
    await Bun.write(outputPath, svgContent);
    generatedCount++;
  }

  console.log(`Generated ${generatedCount} SVGs`);
}

convertDictionaryToSVGs();
