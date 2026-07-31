import os
import re

print("=== Intelligens Kódkereső Ágens Indítása ===")

# 1. Összes Rust fájl megkeresése
rust_files = []
for root, dirs, files in os.walk("src"):
    for file in files:
        if file.endswith(".rs"):
            rust_files.append(os.path.join(root, file))

print(f"\n[1/3] Talált Rust fájlok száma: {len(rust_files)}")

# 2. Kulcsszavak keresése a fájlokban
print("\n[2/3] Struktúrák és tesztek keresése...")
for path in rust_files:
    with open(path, "r", errors="ignore") as f:
        content = f.read()
        
        # Keressünk PolicyEngine vagy TaskBuilder definíciókat
        if "struct PolicyEngine" in content or "impl<'a> PolicyEngine" in content:
            print(f"\n✨ Megtalálva: PolicyEngine helye -> {path}")
            # Írjuk ki az impl blokk elejét
            lines = content.split("\n")
            for i, line in enumerate(lines):
                if "impl" in line and "PolicyEngine" in line:
                    print("\n".join(lines[max(0, i-2):min(len(lines), i+15)]))
                    
        if "struct TaskBuilder" in content or "impl TaskBuilder" in content:
            print(f"\n✨ Megtalálva: TaskBuilder helye -> {path}")
            lines = content.split("\n")
            for i, line in enumerate(lines):
                if "impl" in line and "TaskBuilder" in line:
                    print("\n".join(lines[max(0, i-2):min(len(lines), i+15)]))

# 3. Nézzünk meg egy létező tesztet példának
print("\n[3/3] Meglévő policy teszt keresése minta gyanánt...")
for path in rust_files:
    if "policy" in path and "test" in path or "mod.rs" in path:
        with open(path, "r", errors="ignore") as f:
            content = f.read()
            if "fn test_" in content:
                print(f"\n📖 Tesztpélda forrása: {path}")
                lines = content.split("\n")
                test_start = -1
                for i, line in enumerate(lines):
                    if "fn test_" in line:
                        test_start = i
                        break
                if test_start != -1:
                    print("\n".join(lines[max(0, test_start-1):min(len(lines), test_start+25)]))
                break

print("\n=== Keresés kész. Kérlek, másold vissza a fenti kimenetet! ===")
