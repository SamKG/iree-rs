# Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
# See https://llvm.org/LICENSE.txt for license information.
# SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
# Also available under a BSD-style license. See LICENSE.

import sys

from PIL import Image
import requests

import torch
import torchvision.models as models
from torchvision import transforms

import torch_mlir
from torch_mlir_e2e_test.linalg_on_tensors_backends import refbackend




def load_labels():
    classes_text = requests.get(
        "https://raw.githubusercontent.com/cathyzhyi/ml-data/main/imagenet-classes.txt",
        stream=True,
    ).text
    labels = [line.strip() for line in classes_text.splitlines()]
    return labels


def top3_possibilities(res):
    _, indexes = torch.sort(res, descending=True)
    percentage = torch.nn.functional.softmax(res, dim=1)[0] * 100
    top3 = [(labels[idx], percentage[idx].item()) for idx in indexes[0][:3]]
    return top3


def predictions(torch_func, jit_func, img, labels):
    golden_prediction = top3_possibilities(torch_func(img))
    print("PyTorch prediction")
    print(golden_prediction)
    prediction = top3_possibilities(torch.from_numpy(jit_func(img.numpy())))
    print("torch-mlir prediction")
    print(prediction)


print("load image from " + image_url, file=sys.stderr)
img = load_and_preprocess_image(image_url)
labels = load_labels()

resnet18 = models.resnet18(pretrained=True)
resnet18.train(False)
module = torch_mlir.compile(resnet18, torch.ones(1, 3, 224, 224), output_type="linalg-on-tensors")
backend = refbackend.RefBackendLinalgOnTensorsBackend()
compiled = backend.compile(module)
jit_module = backend.load(compiled)

predictions(resnet18.forward, jit_module.forward, img, labels)
