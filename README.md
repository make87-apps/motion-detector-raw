# Motion Detector Raw

This application subscribes to an incoming `IMAGE_RAW` topic containing uncompressed images in various formats (YUV420, YUV422, YUV444, RGB888, RGBA8888) and performs motion detection using OpenCV's MOG2 background subtraction. When motion is detected, the original input image is published to the `MOTION_IMAGE_RAW` topic.

## Inputs

- **Topic:** `IMAGE_RAW`
- **Message Type:** `make87_messages.image.uncompressed.ImageRawAny`
- **Description:** Receives uncompressed images in multiple formats.

## Outputs

- **Topic:** `MOTION_IMAGE_RAW`
- **Message Type:** `make87_messages.image.uncompressed.ImageRawAny`
- **Description:** Publishes the original input image when motion is detected.

## Config Values

- **PROCESSING_RESCALE_WIDTH**  
  *Width to rescale the image to before motion detection.*  
  Default: `960`

- **MOG2_HISTORY**  
  *History parameter for the MOG2 background subtractor.*  
  Default: `500`

- **MOG2_VAR_THRESHOLD**  
  *Variance threshold for the MOG2 background subtractor.*  
  Default: `16.0`

- **MOG2_DETECT_SHADOWS**  
  *Enable shadow detection for the MOG2 background subtractor (`true`/`false`).*  
  Default: `true`
