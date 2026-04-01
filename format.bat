@echo off
echo Formatting Python code...
black *.py

echo Formatting Rust code...
cargo fmt

echo Linting Python...
flake8 *.py --max-line-length=88 --ignore=E203,W503

echo Linting Rust...
cargo clippy -- -D warnings

echo Done!