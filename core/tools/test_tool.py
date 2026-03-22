import sys
import os

def main():
    print("🐍 [PYTHON-TOOL] Herramienta iniciada en el Sandbox WASM.")
    print(f"📁 Directorio actual: {os.getcwd()}")
    print(f"🆔 Argumentos: {sys.argv[1:]}")
    
    # Simular una operación de herramienta
    if len(sys.argv) > 1:
        op = sys.argv[1]
        print(f"⚙️ Procesando operación: {op}")
        if op == "greet":
            name = sys.argv[2] if len(sys.argv) > 2 else "Sentinel"
            print(f"✨ ¡Hola, {name}! Soy tu herramienta agéntica en Python.")
            
    print("✅ [PYTHON-TOOL] Ejecución completada.")

if __name__ == "__main__":
    main()
