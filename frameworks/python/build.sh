#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

echo "==> Cleaning old build artifacts..."
rm -rf dist/ build/ *.egg-info/ pgl/proto/*_pb2*.py pgl/proto/__pycache__/

echo "==> Generating protobuf files..."
# Ensure grpcio-tools is installed
python3 -m pip install -q grpcio-tools

# Generate the files
python3 -m grpc_tools.protoc \
	-I../../proto \
	--python_out=./pgl/proto \
	--grpc_python_out=./pgl/proto \
	../../proto/pgl_rpc.proto

echo "==> Patching protobuf imports for package compatibility..."
# Fix the absolute import issue in the generated grpc file (Linux/macOS compatible)
if [[ "$OSTYPE" == "darwin"* ]]; then
	sed -i '' 's/import pgl_rpc_pb2 as pgl__rpc__pb2/from \. import pgl_rpc_pb2 as pgl__rpc__pb2/g' pgl/proto/pgl_rpc_pb2_grpc.py
else
	sed -i 's/import pgl_rpc_pb2 as pgl__rpc__pb2/from \. import pgl_rpc_pb2 as pgl__rpc__pb2/g' pgl/proto/pgl_rpc_pb2_grpc.py
fi

echo "==> Building package..."
# Ensure build dependencies are installed
python3 -m pip install -q build twine
python3 -m build

echo "==> Validating package..."
python3 -m twine check dist/*

echo "==> Ready to publish!"
echo "To upload to TestPyPI (recommended first):"
echo "  python3 -m twine upload --repository testpypi dist/*"
echo ""
echo "To upload to production PyPI:"
echo "  python3 -m twine upload dist/*"
