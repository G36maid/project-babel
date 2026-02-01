# Babel Symbols Generator

A procedural SVG glyph generator for a constructed language, inspired by the game *Chants of Sennaar*.

This tool generates distinct abstract symbols for a dictionary of words, including specific handling for ideological concepts (e.g., Capital vs. Communism) and grammatical markers.

## Project Structure

*   `main.py`: The core script that reads words, generates SVG definitions, and outputs both a JSON dictionary and an HTML preview.
*   `words.json`: The source list of words categorised by types (normal, censored/ideologies).
*   `dictionary.json`: The generated output containing mapping from words to SVG XML strings.
*   `preview.html`: A visual grid to preview all generated symbols in a web browser.

## Prerequisites

*   **uv**: This project uses [uv](https://github.com/astral-sh/uv) for Python package and environment management.

## Usage

1.  **Modify Word List (Optional)**
    Edit `words.json` to add or remove words.

2.  **Generate Symbols**
    Run the main script using `uv`:
    ```bash
    uv run main.py
    ```
    This command will:
    *   Read `words.json`.
    *   Generate/Update `dictionary.json`.
    *   Generate/Update `preview.html`.

3.  **Preview**
    Open `preview.html` in your web browser to verify the glyph designs.

## Design Philosophy

*   **Abstract Geometry**: Symbols are built from primitive shapes (lines, circles, paths).
*   **Semantic Distinctions**: Concepts that might be grouped together in other contexts (e.g., *Democracy* vs *Dictatorship*) have distinct, visually metaphoric representations.
*   **Standardized Format**: All glyphs are generated on a 100x100 SVG coordinate system.
