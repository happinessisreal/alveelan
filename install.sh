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

# 2. Build in release mode
echo "কোড তৈরি (Build) করা হচ্ছে..."
cargo build --release

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
