import json

# Analyzes text in a document stored in an S3 bucket. Display polygon box around text and angled text
import boto3
import io
from io import BytesIO
import sys
import base64
import math
import cv2
import numpy as np
from PIL import Image

# from PIL.Image import core as _imaging


def thresh_check(res):
    for v1 in res:
        for v2 in v1:
            if v2 > 0.9:
                return True

    return False


def lambda_handler(request):
    print(request.get_data())

    bucket = "wordbites-data"
    document = "test_small.jpg"
    # blocks=process_text_analysis(bucket,document)
    # return {
    #     'statusCode': 200,
    #     'body': json.dumps(event['body']),
    # }

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
                        # wtexts.append(block['Text'] + block2['Text'])
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
    # singles = "NONE" if not stexts else str(stexts)
    # heights = "NONE" if not htexts else str(htexts)
    # widths = "NONE" if not wtexts  else str(wtexts)
    # print(singles)
    # print(heights)
    # print(widths)
    # , "blocks": json.dumps(str(blocks))
    stexts = [text.replace(" ", "") for text in stexts]
    wtexts = [text.replace(" ", "") for text in wtexts]
    htexts = [text.replace(" ", "") for text in htexts]
    return {
        "statusCode": 200,
        "body": json.dumps({"single": stexts, "tall": htexts, "wide": wtexts}),
    }


def ShowBoundingBox(draw, box, width, height, boxColor):

    left = width * box["Left"]
    top = height * box["Top"]
    draw.rectangle(
        [left, top, left + (width * box["Width"]), top + (height * box["Height"])],
        outline=boxColor,
    )


def ShowSelectedElement(draw, box, width, height, boxColor):

    left = width * box["Left"]
    top = height * box["Top"]
    draw.rectangle(
        [left, top, left + (width * box["Width"]), top + (height * box["Height"])],
        fill=boxColor,
    )


# Displays information about a block returned by text detection and text analysis
def DisplayBlockInformation(block):
    print("Id: {}".format(block["Id"]))
    if "Text" in block:
        print("    Detected: " + block["Text"])
    print("    Type: " + block["BlockType"])

    if "Confidence" in block:
        print("    Confidence: " + "{:.2f}".format(block["Confidence"]) + "%")

    if block["BlockType"] == "CELL":
        print("    Cell information")
        print("        Column:" + str(block["ColumnIndex"]))
        print("        Row:" + str(block["RowIndex"]))
        print("        Column Span:" + str(block["ColumnSpan"]))
        print("        RowSpan:" + str(block["ColumnSpan"]))

    if "Relationships" in block:
        print("    Relationships: {}".format(block["Relationships"]))
    print("    Geometry: ")
    print("        Bounding Box: {}".format(block["Geometry"]["BoundingBox"]))
    print("        Polygon: {}".format(block["Geometry"]["Polygon"]))

    if block["BlockType"] == "KEY_VALUE_SET":
        print("    Entity Type: " + block["EntityTypes"][0])

    if block["BlockType"] == "SELECTION_ELEMENT":
        print("    Selection element detected: ", end="")

        if block["SelectionStatus"] == "SELECTED":
            print("Selected")
        else:
            print("Not selected")

    if "Page" in block:
        print("Page: " + block["Page"])
    print()


def process_request(body):

    # Get the document from S3
    # s3_connection = boto3.resource('s3')

    # s3_object = s3_connection.Object(bucket,document)
    # s3_response = s3_object.get()

    # stream = io.BytesIO(s3_response['Body'].read())
    # image=Image.open(stream)

    print(body)
    temp = base64.b64decode(body)
    stream = io.BytesIO(temp)

    image=Image.open(stream)
    org_image = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2BGR)
    # org_image = cv2.cvtColor(test, cv2.COLOR_RGB2BGR)

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

    # stream = io.BytesIO(org_image)
    stream = cv2.imencode('.jpg', org_image)[1].tobytes()
    print(base64.b64encode(stream))
    # image=Image.open(stream)
    # Analyze the document
    client = boto3.client('textract')

    # image_binary = stream.getvalue()
    response = client.analyze_document(
        Document={"Bytes": stream}, FeatureTypes=["TABLES", "FORMS"]
    )

    # Alternatively, process using S3 object
    # response = client.analyze_document(
    #    Document={'S3Object': {'Bucket': bucket, 'Name': document}},
    #    FeatureTypes=["TABLES", "FORMS"])

    # Get the text blocks
    blocks = response["Blocks"]
    # width, height = org_image.size
    # draw = ImageDraw.Draw(image)
    # print("Detected Document Text")

    # Create image showing bounding box/polygon the detected lines/text
    # for block in blocks:

    #     DisplayBlockInformation(block)

        # Uncomment to draw block
        # draw=ImageDraw.Draw(image)
        # if block['BlockType'] == "KEY_VALUE_SET":
        #     if block['EntityTypes'][0] == "KEY":
        #         ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height,'red')
        #     else:
        #         ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height,'green')

        # if block['BlockType'] == 'TABLE':
        #     ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height, 'blue')

        # if block['BlockType'] == 'CELL':
        #     ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height, 'yellow')
        # if block['BlockType'] == 'SELECTION_ELEMENT':
        #     if block['SelectionStatus'] =='SELECTED':
        #         ShowSelectedElement(draw, block['Geometry']['BoundingBox'],width,height, 'blue')

        # uncomment to draw polygon for all Blocks
        # points=[]
        # for polygon in block['Geometry']['Polygon']:
        #    points.append((width * polygon['X'], height * polygon['Y']))
        # draw.polygon((points), outline='blue')

    # Display the image
    # image.show()
    return blocks


# def process_text_analysis(bucket, document):

#     #Get the document from S3
#     s3_connection = boto3.resource('s3')

#     s3_object = s3_connection.Object(bucket,document)
#     s3_response = s3_object.get()

#     stream = io.BytesIO(s3_response['Body'].read())
#     image=Image.open(stream)

#     # Analyze the document
#     client = boto3.client('textract')

#     image_binary = stream.getvalue()
#     response = client.analyze_document(Document={'Bytes': image_binary},
#         FeatureTypes=["TABLES", "FORMS"])


#     # Alternatively, process using S3 object
#     #response = client.analyze_document(
#     #    Document={'S3Object': {'Bucket': bucket, 'Name': document}},
#     #    FeatureTypes=["TABLES", "FORMS"])


#     #Get the text blocks
#     blocks=response['Blocks']
#     width, height =image.size
#     draw = ImageDraw.Draw(image)
#     print ('Detected Document Text')

#     # Create image showing bounding box/polygon the detected lines/text
#     for block in blocks:

#         DisplayBlockInformation(block)

#         #Uncomment to draw block
#         # draw=ImageDraw.Draw(image)
#         # if block['BlockType'] == "KEY_VALUE_SET":
#         #     if block['EntityTypes'][0] == "KEY":
#         #         ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height,'red')
#         #     else:
#         #         ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height,'green')

#         # if block['BlockType'] == 'TABLE':
#         #     ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height, 'blue')

#         # if block['BlockType'] == 'CELL':
#         #     ShowBoundingBox(draw, block['Geometry']['BoundingBox'],width,height, 'yellow')
#         # if block['BlockType'] == 'SELECTION_ELEMENT':
#         #     if block['SelectionStatus'] =='SELECTED':
#         #         ShowSelectedElement(draw, block['Geometry']['BoundingBox'],width,height, 'blue')

#             #uncomment to draw polygon for all Blocks
#             #points=[]
#             #for polygon in block['Geometry']['Polygon']:
#             #    points.append((width * polygon['X'], height * polygon['Y']))
#             #draw.polygon((points), outline='blue')

#     # Display the image
#     #image.show()
#     return blocks


# def main():

#     bucket = 'wordbites-data'
#     document = 'test.jpg'
#     block_count=process_text_analysis(bucket,document)
#     print("Blocks detected: " + str(block_count))


# if __name__ == "__main__":
#     lambda_handler()

#   """Responds to any HTTP request.
#   Args:
#       request (flask.Request): HTTP request object.
#   Returns:
#       The response text or any set of values that can be turned into a
#       Response object using
#       `make_response <http://flask.pocoo.org/docs/1.0/api/#flask.Flask.make_response>`.
#   """
# request_json = request.get_json()
# if request.args and 'message' in request.args:
#   return request.args.get('message')
# elif request_json and 'message' in request_json:
#   return request_json['message']
# else:
#   return f'Hello World!'
