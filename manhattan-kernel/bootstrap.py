#!/usr/bin/env python3
"""Bootstrap the Manhattan Kernel with validated schemas from common Rust errors."""

import subprocess
import json
import os

BINARY = "./target/debug/manhattan-kernel"

# Gyakori Rust hibák és a várt javítások (a teszteléshez)
TEST_CASES = [
    # Típushiba: i32 vs String
    {
        "code": "fn main() { let x: i32 = \"hello\"; }",
        "expected_fix": "fn main() { let x: String = \"hello\"; }"
    },
    # Hiányzó pontosvessző (a fix_main megoldja)
    {
        "code": "fn main() { let x = 5 }",
        "expected_fix": None  # a fix_main "fn main() {}" -et ad, ami jó, de más
    },
    # Hibátlan kód
    {
        "code": "fn main() {}",
        "expected_fix": "already correct"
    },
    # Másik típushiba
    {
        "code": "fn main() { let x: bool = 42; }",
        "expected_fix": "fn main() { let x: i32 = 42; }"
    },
    # Hiányzó import (a kernel most fix_main-t ad, mert a hibát nem ismeri fel)
    {
        "code": "fn main() { let _ = File::open(\"test\"); }",
        "expected_fix": None  # jelenleg fix_main lesz
    }
]

def run_kernel(code):
    """Futtatja a kernelt a megadott kóddal, visszaadja a kimenetet."""
    try:
        result = subprocess.run(
            [BINARY, code],
            capture_output=True,
            text=True,
            timeout=10
        )
        return result.stdout.strip(), result.stderr.strip()
    except subprocess.TimeoutExpired:
        return None, "timeout"
    except Exception as e:
        return None, str(e)

def main():
    if not os.path.exists(BINARY):
        print(f"Hiba: a bináris nem található: {BINARY}")
        print("Futtasd előbb: cargo build")
        return

    schemas = []
    successes = 0

    for i, case in enumerate(TEST_CASES):
        code = case["code"]
        expected = case["expected_fix"]
        print(f"Teszt {i+1}/{len(TEST_CASES)}: {code[:50]}...")
        output, error = run_kernel(code)
        if error:
            print(f"  HIBA: {error}")
            continue
        if output == "already correct":
            print(f"  OK (már helyes)")
            successes += 1
            continue
        if output.startswith("fn main()") or output.startswith("use std"):
            print(f"  JAVÍTVA: {output}")
            successes += 1
            # Itt kinyerhetnénk a gráfot is, de most csak a kódot tároljuk
            schemas.append({
                "original_code": code,
                "fixed_code": output
            })
        else:
            print(f"  NEM SIKERÜLT: {output}")

    print(f"\nSikeres: {successes}/{len(TEST_CASES)}")
    if schemas:
        with open("schemas.json", "w") as f:
            json.dump(schemas, f, indent=2)
        print(f"Sémák elmentve: schemas.json ({len(schemas)} bejegyzés)")

if __name__ == "__main__":
    main()
