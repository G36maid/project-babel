import json
import os

# ==========================================
# 1. 字典生成邏輯 (Dictionary Generation)
# ==========================================

# --- SVG 繪圖輔助函式 ---
def svg_line(x1, y1, x2, y2, width=3):
    return f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="currentColor" stroke-width="{width}" stroke-linecap="round" />'

def svg_circle(cx, cy, r, filled=False):
    fill = "currentColor" if filled else "none"
    return f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="currentColor" stroke-width="3" />'

def svg_rect(x, y, w, h, filled=False):
    fill = "currentColor" if filled else "none"
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" stroke="currentColor" stroke-width="3" />'

def svg_path(d, fill="none"):
    return f'<path d="{d}" fill="{fill}" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />'

def svg_arrow(x, y, direction="right", size=15):
    if direction == "right":
        return f'<path d="M {x} {y} L {x-size} {y-size} M {x} {y} L {x-size} {y+size}" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" />'
    elif direction == "left":
        return f'<path d="M {x} {y} L {x+size} {y-size} M {x} {y} L {x+size} {y+size}" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" />'
    elif direction == "up":
        return f'<path d="M {x} {y} L {x-size} {y+size} M {x} {y} L {x+size} {y+size}" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" />'
    elif direction == "down":
        return f'<path d="M {x} {y} L {x-size} {y-size} M {x} {y} L {x+size} {y-size}" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" />'
    return ""

# --- 符號定義 ---
def get_glyph_content(key):
    person_base = svg_line(50, 35, 50, 90) + svg_circle(50, 20, 8, filled=True)
    
    match key:
        # === Pronouns & Entities ===
        case "person": return person_base
        case "me":     return person_base + svg_path("M 30 50 L 20 40 M 30 50 L 20 60")
        case "you":    return person_base + svg_path("M 70 50 L 60 40 M 70 50 L 60 60")
        case "he":     return person_base + svg_path("M 80 20 L 70 10 M 80 20 L 90 10")
        case "god":    return svg_circle(50, 30, 20) + svg_circle(50, 30, 5, filled=True) + svg_path("M 20 80 L 50 45 L 80 80")
        case "unit":   return svg_line(50, 15, 50, 85, width=4)

        # === Verbs ===
        case "say":    return svg_path("M 30 50 Q 50 30 70 50 Q 50 70 30 50") + svg_line(40, 50, 60, 50)
        case "is" | "equal": return svg_line(20, 40, 80, 40) + svg_line(20, 60, 80, 60)
        case "belong": return svg_line(20, 50, 70, 50) + svg_path("M 70 50 L 55 35 M 70 50 L 55 65")
        case "have":   return svg_path("M 25 40 L 25 70 Q 50 90 75 70 L 75 40")
        case "pray":   return svg_line(50, 50, 50, 90) + svg_path("M 50 50 L 20 20 M 50 50 L 80 20")
        case "obey":   return svg_path("M 30 30 L 30 80 L 80 80")
        case "block":  return svg_rect(15, 15, 70, 70) + svg_line(15, 15, 85, 85) + svg_line(85, 15, 15, 85)
        case "change": return svg_path("M 50 20 L 80 80 L 20 80 Z")
        case "hello":  return person_base + svg_path("M 75 25 Q 90 25 90 45")

        # === Logic & Values ===
        case "not":    return svg_line(80, 20, 20, 80, width=4)
        case "or":     return svg_path("M 20 20 L 50 80 L 80 20")
        case "and":    return svg_path("M 20 80 L 50 20 L 80 80 M 35 50 L 65 50")
        case "gt":     return svg_path("M 25 25 L 75 50 L 25 75")
        case "lt":     return svg_path("M 75 25 L 25 50 L 75 75")
        case "one":    return svg_circle(50, 50, 10, filled=True)
        case "few":    return svg_circle(30, 50, 8, filled=True) + svg_circle(70, 50, 8, filled=True)
        case "many":   return svg_circle(50, 25, 8, filled=True) + svg_circle(25, 75, 8, filled=True) + svg_circle(75, 75, 8, filled=True)
        case "value":  return svg_rect(25, 25, 50, 50)
        case "power":  return svg_path("M 55 10 L 35 50 L 65 50 L 45 90")
        case "want":   return svg_path("M 20 30 L 20 70 Q 50 95 80 70 L 80 30") + svg_path("M 30 50 L 70 50")
        case "qmark":  return svg_path("M 35 30 C 35 10, 65 10, 65 40 C 65 60, 50 60, 50 75") + svg_circle(50, 88, 5, filled=True)
        case "what":   return svg_circle(40, 40, 25) + svg_line(60, 60, 85, 85, width=6) # Magnifying glass

        # === Ideologies & Concepts ===
        case "capital":     return svg_path("M 20 80 L 80 80 L 50 35 Z") + svg_circle(50, 20, 6, filled=True)
        case "communism":   return svg_rect(25, 25, 20, 20, filled=True) + svg_rect(55, 25, 20, 20, filled=True) + svg_rect(25, 55, 20, 20, filled=True) + svg_rect(55, 55, 20, 20, filled=True)
        case "democracy":   return svg_path("M 10 70 Q 50 10 90 70") + svg_circle(20, 80, 5, filled=True) + svg_circle(40, 80, 5, filled=True) + svg_circle(60, 80, 5, filled=True) + svg_circle(80, 80, 5, filled=True)
        case "oligarchy":   return svg_circle(50, 35, 15) + svg_circle(35, 65, 15) + svg_circle(65, 65, 15)
        case "dictatorship":return svg_path("M 20 20 L 80 20 L 50 70 Z", fill="currentColor") + svg_line(20, 85, 80, 85, width=5)
        case "tradition":   return svg_rect(15, 15, 70, 70) + svg_rect(35, 35, 30, 30, filled=True)
        case "progress":    return svg_path("M 20 80 L 50 50 L 50 80 M 50 50 L 80 20 L 80 50") + svg_path("M 80 20 L 80 20 M 80 20 L 80 20")
        case "local":       return svg_circle(50, 50, 35) + svg_circle(50, 50, 10, filled=True)
        case "global":      return svg_circle(50, 50, 35) + svg_path("M 50 15 L 50 85 M 15 50 L 85 50") + svg_circle(50, 50, 45)
        case "freedom":     return svg_path("M 30 30 L 10 10") + svg_path("M 70 30 L 90 10") + svg_path("M 30 70 L 10 90") + svg_path("M 70 70 L 90 90") + svg_circle(50, 50, 15)
        case "limit":       return svg_line(30, 20, 30, 80, width=5) + svg_line(70, 20, 70, 80, width=5) + svg_circle(50, 50, 8, filled=True)
        case "equality":    return svg_line(50, 80, 50, 30) + svg_line(20, 30, 80, 30) + svg_circle(20, 40, 5, filled=True) + svg_circle(80, 40, 5, filled=True)
        case "class":       return svg_path("M 20 80 L 40 80 L 40 60 L 60 60 L 60 40 L 80 40 L 80 20")
        case "material":    return svg_path("M 50 20 L 80 35 L 80 75 L 50 90 L 20 75 L 20 35 Z") + svg_path("M 50 50 L 50 90 M 50 50 L 20 35 M 50 50 L 80 35")
        case "deity":       return svg_circle(50, 50, 10, filled=True) + svg_line(50, 30, 50, 10) + svg_line(50, 70, 50, 90) + svg_line(30, 50, 10, 50) + svg_line(70, 50, 90, 50) + svg_line(35, 35, 20, 20) + svg_line(65, 65, 80, 80) + svg_line(35, 65, 20, 80) + svg_line(65, 35, 80, 20)
            
        case _:
            return svg_rect(10, 10, 80, 80) + svg_path("M 30 30 L 70 70 M 70 30 L 30 70")

def generate_dictionary():
    print("--- 1. Generating dictionary.json ---")
    if not os.path.exists("words.json"):
        print("Error: words.json not found!")
        return False

    with open("words.json", "r", encoding="utf-8") as f:
        words_data = json.load(f)

    all_words = []
    all_words.extend(words_data.get("normal", []))
    for group in words_data.get("censored", []):
        if isinstance(group, list):
            all_words.extend(group)
        else:
            all_words.append(group)

    unique_words = list(dict.fromkeys(all_words))
    output_dict = {}
    for word in unique_words:
        output_dict[word] = get_glyph_content(word)

    with open("dictionary.json", "w", encoding="utf-8") as f:
        json.dump(output_dict, f, indent=2, ensure_ascii=False)
    
    print(f"Successfully processed {len(output_dict)} words.")
    return True

# ==========================================
# 2. 預覽生成邏輯 (Preview Generation)
# ==========================================

def generate_preview():
    print("--- 2. Generating preview.html ---")
    try:
        with open("dictionary.json", "r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        print("Error: dictionary.json not found.")
        return False

    html_content = """<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Babel Symbols Preview</title>
    <style>
        body {
            font-family: 'Courier New', Courier, monospace;
            background-color: #1e1e1e;
            color: #d4d4d4;
            margin: 0;
            padding: 20px;
        }
        h1 { text-align: center; color: #9cdcfe; }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
            gap: 20px;
            max-width: 1200px;
            margin: 0 auto;
        }
        .card {
            background-color: #252526;
            border: 1px solid #3c3c3c;
            border-radius: 8px;
            padding: 15px;
            display: flex;
            flex-direction: column;
            align-items: center;
            transition: transform 0.2s;
        }
        .card:hover {
            transform: translateY(-5px);
            border-color: #007acc;
            box-shadow: 0 4px 12px rgba(0,0,0,0.5);
        }
        .symbol-container {
            width: 100px;
            height: 100px;
            background-color: #000;
            border: 1px solid #555;
            border-radius: 4px;
            margin-bottom: 10px;
            color: white; 
        }
        svg {
            width: 100%;
            height: 100%;
            stroke: currentColor;
            stroke-width: 3;
            stroke-linecap: round;
            stroke-linejoin: round;
            fill: none;
        }
        .label {
            font-size: 14px;
            font-weight: bold;
            text-align: center;
            color: #ce9178;
            word-wrap: break-word;
            width: 100%;
        }
        .meta {
            margin-top: 20px;
            text-align: center;
            color: #6a9955;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <h1>Babel Symbols Preview</h1>
    <div class="meta">Generated from dictionary.json</div>
    <br>
    <div class="grid">
"""

    sorted_keys = sorted(data.keys())
    for key in sorted_keys:
        svg_inner = data[key]
        svg_element = f'<svg viewBox="0 0 100 100">{svg_inner}</svg>'
        
        card = f"""
        <div class="card">
            <div class="symbol-container">
                {svg_element}
            </div>
            <div class="label">{key}</div>
        </div>
        """
        html_content += card

    html_content += """
    </div>
</body>
</html>
"""

    with open("preview.html", "w", encoding="utf-8") as f:
        f.write(html_content)
    
    print(f"Preview generated: {os.path.abspath('preview.html')}")
    return True

# ==========================================
# Main Execution
# ==========================================

if __name__ == "__main__":
    if generate_dictionary():
        generate_preview()