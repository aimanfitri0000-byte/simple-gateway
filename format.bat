@echo off
echo Formatting Python code...
python -m black mock_service.py mock_service2.py

echo.
echo Formatting Rust code...
cargo fmt

echo.
echo Linting Python...
python -m flake8 mock_service.py mock_service2.py --max-line-length=88 --ignore=E203,W503

echo.
echo Linting Rust...
cargo clippy -- -D warnings

echo.
echo Done!