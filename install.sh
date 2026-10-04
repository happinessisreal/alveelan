#!/bin/bash

# Alveelan Installer
set -e

echo "------------------------------------------"
echo "আলভীলান (Alveelan) কম্পাইলার ইন্সটল হচ্ছে..."
echo "------------------------------------------"

# 1. Check dependencies
if ! command -v cargo &> /dev/null; then
    echo "ত্রুটি: Rust (cargo) পাওয়া যায়নি। দয়া করে https://rustup.rs থেকে ইন্সটল করুন।"
    exit 1
fi

if ! command -v cc &> /dev/null; then
    echo "সতর্কতা: 'cc' লিঙ্ককার পাওয়া যায়নি। কোড কম্পাইল করতে এটি প্রয়োজন হবে।"
fi

# 2. Detect a supported LLVM (18–21) and build in release mode
FEATURES=""
for v in 21 20 19 18; do
    for cfg in "llvm-config-$v" "llvm-config"; do
        if command -v "$cfg" &> /dev/null && [ "$("$cfg" --version | cut -d. -f1)" = "$v" ]; then
            FEATURES="--no-default-features --features llvm$v"
            export "LLVM_SYS_${v}1_PREFIX=$("$cfg" --prefix)"
            echo "LLVM $v পাওয়া গেছে ($cfg)।"
            break 2
        fi
    done
done
if [ -z "$FEATURES" ]; then
    echo "সতর্কতা: LLVM 18–21 পাওয়া যায়নি। LLVM 18 ধরে নিয়ে চেষ্টা করা হচ্ছে।"
    echo "(Warning: no LLVM 18–21 found; install e.g. 'llvm-18-dev' and retry if the build fails.)"
fi

echo "কোড তৈরি (Build) করা হচ্ছে..."
# shellcheck disable=SC2086
cargo build --release $FEATURES

# 3. Determine install directory
INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.alveelan/bin"
    mkdir -p "$INSTALL_DIR"
fi

# 4. Copy binary
echo "বাইনারি কপি করা হচ্ছে: $INSTALL_DIR/alveelan"
cp target/release/alveelan "$INSTALL_DIR/alveelan"
chmod +x "$INSTALL_DIR/alveelan"

echo ""
echo "------------------------------------------"
echo "অভিনন্দন! আলভীলান ইন্সটল সফল হয়েছে।"
echo "ইন্সটল ডিরেক্টরি: $INSTALL_DIR"
echo ""
echo "কোথাও থেকে আলভীলান চালাতে আপনার PATH এ এটি যোগ করুন:"
echo "export PATH=\"\$PATH:$INSTALL_DIR\""
echo "------------------------------------------"
