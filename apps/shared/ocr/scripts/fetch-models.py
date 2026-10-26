#!/usr/bin/env python3
"""Download the pinned PP-OCRv5 ONNX models into models/ and extract their dictionaries."""

import argparse
import hashlib
import json
import shutil
import sys
import urllib.request
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
HF = "https://huggingface.co/PaddlePaddle/{repo}/resolve/{rev}/{file}"

DETECTOR = (
    "PP-OCRv5_mobile_det_onnx",
    "e6f4fa85f00e168c862bc462aebca69eef9b3d3d",
    "a431985659dc921974177a95adcfbb90fd9e51989a5e04d70d0b75f597b6e61d",
)

RECOGNIZERS = {
    "korean": ("korean_PP-OCRv5_mobile_rec_onnx", "5c6f574b8e2230adf4287b33e736d71b9fabd28e", "92f0b7785e64fc9090106a241cf4c1eb97472824558272751b88a2a4476d3a08"),
    "chinese": ("PP-OCRv5_mobile_rec_onnx", "ed152b8b495f84de93cda5709d768548a9127622", "da72dc72ca4dc220df0dfde68c1dedc31c58d3e76a25871122e5056227d50092"),
    "latin": ("latin_PP-OCRv5_mobile_rec_onnx", "89d3a50e2c27e2e7cceeab0e944c25c807d5db4f", "7888113072263cb471b93f66dd5e2ad70548dc526fa1ace760d0d973dd121498"),
    "cyrillic": ("cyrillic_PP-OCRv5_mobile_rec_onnx", "2cef88145434beb8afa9dd82d77d799eb1ad7b29", "5371ee1ddaa7983cc62d0818d99e982b6804638c85e4f960d59a574094e172e5"),
    "eslav": ("eslav_PP-OCRv5_mobile_rec_onnx", "9a32171fc5718746875e1a261818884517975013", "b3018ef2b09a0250b6e0c8e871c927098363e5fd4df890cc68e8358eb0aaf1bd"),
    "arabic": ("arabic_PP-OCRv5_mobile_rec_onnx", "14aaedcd75825982689ecf5cd64ab33ee083215a", "799113ebf267fbe742deb99eb36e8d42c9ddc5291ceacf92add41b4d52a59110"),
    "devanagari": ("devanagari_PP-OCRv5_mobile_rec_onnx", "251aec19e36739540d35e2cc943f6aa7503b98e5", "cb789212ce96c69d3e74728ae4309d179281d68cb3945d0616b67cafab41c986"),
    "greek": ("el_PP-OCRv5_mobile_rec_onnx", "8152b89d2ee0e1d4c92decab75fe75e5e3d836b6", "2acf17fcaea2bc81b878e311e6263b8885f48bb03796f75f9f30ed3242bbaa6d"),
    "thai": ("th_PP-OCRv5_mobile_rec_onnx", "1d4adbbafb1034a2fd6618498575b81ea7b69f69", "27618be66018f8598ac0a526a593f9f1cebf794e7eded93428e8fb016e537f5f"),
    "tamil": ("ta_PP-OCRv5_mobile_rec_onnx", "0db89798af0d5218fad9ea7ace0557526e9077f1", "c6d2b682d2a0ea4cb1fccdba295976f93fd439964d16cdc666cadef531accbee"),
    "telugu": ("te_PP-OCRv5_mobile_rec_onnx", "f1197c83430241c58c64b0952b3a80366d04eeba", "8238bfc46d4cffe720ed6706e3842802467343497428693ff2bfb4e6b3caa36b"),
    "english": ("en_PP-OCRv5_mobile_rec_onnx", "3fafbc3b5dcf93dd72add9f48368be8a3a2cd33b", "b5f833dfc5d0eb71da397b4efa06ebeee9b431b690a47d6af40d77d8eabc557f"),
}


def fetch(repo: str, rev: str, file: str) -> bytes:
    with urllib.request.urlopen(HF.format(repo=repo, rev=rev, file=file)) as response:
        return response.read()


def fetch_onnx(repo: str, rev: str, sha256: str, dest: Path) -> None:
    if dest.exists() and hashlib.sha256(dest.read_bytes()).hexdigest() == sha256:
        return
    data = fetch(repo, rev, "inference.onnx")
    actual = hashlib.sha256(data).hexdigest()
    if actual != sha256:
        sys.exit(f"{repo}: sha256 mismatch, expected {sha256}, got {actual}")
    dest.write_bytes(data)


def fetch_dict(repo: str, rev: str, dest: Path) -> None:
    config = yaml.safe_load(fetch(repo, rev, "inference.yml"))
    chars = [str(c) for c in config["PostProcess"]["character_dict"]]
    if any("\n" in c for c in chars):
        sys.exit(f"{repo}: dictionary entry contains a newline")
    dest.write_text("\n".join(chars) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT / "models")
    parser.add_argument("scripts", nargs="*", default=list(RECOGNIZERS))
    args = parser.parse_args()

    unknown = set(args.scripts) - set(RECOGNIZERS)
    if unknown:
        sys.exit(f"unknown scripts: {', '.join(sorted(unknown))}")

    (args.out / "rec").mkdir(parents=True, exist_ok=True)
    fetch_onnx(*DETECTOR, args.out / "det.onnx")
    for script in args.scripts:
        repo, rev, sha256 = RECOGNIZERS[script]
        print(f"fetching {script}", file=sys.stderr)
        fetch_onnx(repo, rev, sha256, args.out / "rec" / f"{script}.onnx")
        fetch_dict(repo, rev, args.out / "rec" / f"{script}.txt")

    for notice in ("NOTICE", "LICENSE-MODELS"):
        shutil.copyfile(ROOT / notice, args.out / notice)

    manifest = {"scripts": args.scripts}
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
