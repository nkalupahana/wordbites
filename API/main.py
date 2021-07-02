import json
import boto3
import io
from io import BytesIO
import sys
import base64
import math
import cv2
import numpy as np
from PIL import Image

def thresh_check(res):
    for v1 in res:
        for v2 in v1:
            if v2 > 0.9:
                return True

    return False

def lambda_handler(request):
    bucket = "wordbites-data"
    document = "test_small.jpg"

    decodedImage = base64.b64decode(request.get_data())
    blocks = process_request(request.get_data())
    scoreBlock = {}
    stexts = []
    htexts = []
    wtexts = []
    for block in blocks:
        if block["BlockType"] == "LINE":
            if "score" in block["Text"].lower():
                scoreBlock = block
    for block in blocks:
        if block["BlockType"] == "LINE":
            for block2 in blocks:
                if block2["BlockType"] == "LINE" and block2 != block:
                    if (
                        abs(
                            block["Geometry"]["BoundingBox"]["Top"]
                            + block["Geometry"]["BoundingBox"]["Height"]
                            - block2["Geometry"]["BoundingBox"]["Top"]
                        )
                        < 0.05
                        and abs(
                            block["Geometry"]["BoundingBox"]["Left"]
                            - block2["Geometry"]["BoundingBox"]["Left"]
                        )
                        < 0.05
                    ):
                        temp = block["Text"] + block2["Text"]
                        if (
                            len(temp.replace(" ", "")) <= 2
                            and len(temp.replace(" ", "")) > 0
                            and block["Geometry"]["BoundingBox"]["Top"]
                            > scoreBlock["Geometry"]["BoundingBox"]["Top"]
                        ):
                            htexts.append(temp)
                            block["Text"] = ""
                            block2["Text"] = ""
                    if (
                        abs(
                            block["Geometry"]["BoundingBox"]["Left"]
                            + block["Geometry"]["BoundingBox"]["Width"]
                            - block2["Geometry"]["BoundingBox"]["Left"]
                        )
                        < 0.1
                        and abs(
                            block["Geometry"]["BoundingBox"]["Top"]
                            - block2["Geometry"]["BoundingBox"]["Top"]
                        )
                        < 0.1
                    ):
                        temp = block["Text"] + block2["Text"]
                        if (
                            len(temp.replace(" ", "")) <= 2
                            and len(temp.replace(" ", "")) > 0
                            and block["Geometry"]["BoundingBox"]["Top"]
                            > scoreBlock["Geometry"]["BoundingBox"]["Top"]
                        ):
                            wtexts.append(temp)
                            block["Text"] = ""
                            block2["Text"] = ""

    for block in blocks:
        if block["BlockType"] == "LINE":
            if (
                len(block["Text"].replace(" ", "")) <= 2
                and len(block["Text"].replace(" ", "")) > 0
                and block["Geometry"]["BoundingBox"]["Top"]
                > scoreBlock["Geometry"]["BoundingBox"]["Top"]
            ):
                if len(block["Text"].strip()) == 1:
                    stexts.append(block["Text"])
                elif "Geometry" in block:
                    if "BoundingBox" in block["Geometry"]:
                        if (
                            block["Geometry"]["BoundingBox"]["Width"]
                            > block["Geometry"]["BoundingBox"]["Height"]
                            and block["Text"] not in wtexts
                        ):
                            wtexts.append(block["Text"])
                        else:
                            if (
                                block["Geometry"]["BoundingBox"]["Width"]
                                <= block["Geometry"]["BoundingBox"]["Height"]
                                and block["Text"] not in htexts
                            ):
                                htexts.append(block["Text"])
    print("Blocks detected: " + str(blocks))

    stexts = [text.replace(" ", "") for text in stexts]
    wtexts = [text.replace(" ", "") for text in wtexts]
    htexts = [text.replace(" ", "") for text in htexts]


    if request.method == 'OPTIONS':
        # Allows GET requests from any origin with the Content-Type
        # header and caches preflight response for an 3600s
        headers = {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'POST',
            'Access-Control-Allow-Headers': 'Content-Type',
            'Access-Control-Max-Age': '3600'
        }

        return ('', 204, headers)
    headers = {
        'Access-Control-Allow-Origin': '*',
    }

    return ({"single": stexts, "tall": htexts, "wide": wtexts}, 200, headers)

def process_request(body):

    temp = base64.b64decode(body)
    stream = io.BytesIO(temp)

    image=Image.open(stream)
    org_image = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2BGR)

    template1 = cv2.imread("I.jpg")
    template = cv2.cvtColor(template1, cv2.COLOR_BGR2GRAY)
    w, h = template.shape[::-1]

    s_img = cv2.imread("BIG_I.jpg")
    count = 0

    while True:
        res = cv2.matchTemplate(
            cv2.cvtColor(org_image, cv2.COLOR_BGR2GRAY), template, cv2.TM_CCOEFF_NORMED
        )
        if not thresh_check(res):
            break

        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(res)

        top_left = max_loc
        bottom_right = (top_left[0] + w, top_left[1] + h)

        org_image[top_left[1] : bottom_right[1], top_left[0] : bottom_right[0]] = s_img
        print(count)
        count += 1

    stream = cv2.imencode('.jpg', org_image)[1].tobytes()
    print(base64.b64encode(stream))
    client = boto3.client('textract')

    response = client.analyze_document(
        Document={"Bytes": stream}, FeatureTypes=["TABLES", "FORMS"]
    )
    blocks = response["Blocks"]
    return blocks
