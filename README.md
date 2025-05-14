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

## Config Values & Recommendations

| Parameter                  | Recommendation & Use Case                                                                                   |
|----------------------------|------------------------------------------------------------------------------------------------------------|
| **PROCESSING_RESCALE_WIDTH** | Width to rescale the image before motion detection.<br>**960** is a good default for most scenes (fast, detects people/vehicles). |
| **MOG2_HISTORY**           | History parameter for the MOG2 background subtractor.<br>**500** is a good balance for scenes that change slowly. |
| **MOG2_VAR_THRESHOLD**     | Variance threshold for the MOG2 background subtractor.<br>**8–16** for indoor, **16–24** for outdoor stable, **24–36** for outdoor with trees/shadows, **36–48** for high-noise scenes. |
| **MOG2_DETECT_SHADOWS**    | Enable shadow detection for the MOG2 background subtractor (`true`/`false`).<br>**true** recommended for most use cases. |
| **MOTION_PIXEL_THRESHOLD**  | Minimum number of changed pixels in the mask to trigger motion detection.<br>**300–800** for indoor, **500–1500** for outdoor stable, **800–2000** for outdoor with trees/shadows, **1500–4000** for high-noise scenes. |

### Example Use Cases

| Use Case / Scene Type                             | `MOG2_VAR_THRESHOLD` | `MOTION_PIXEL_THRESHOLD` | Notes                                                       |
| ------------------------------------------------- | -------------------- | ------------------------ | ----------------------------------------------------------- |
| **Indoor, controlled lighting**                   | `8.0 – 16.0`         | `300 – 800`              | Detects small changes like a person entering a hallway      |
| **Outdoor, stable background (low noise)**        | `16.0 – 24.0`        | `500 – 1500`             | Good for typical street surveillance (your case)            |
| **Outdoor, variable background (trees, shadows)** | `24.0 – 36.0`        | `800 – 2000`             | Reduces false positives from leaves, light shifts           |
| **High-noise scene (e.g., traffic + foliage)**    | `36.0 – 48.0`        | `1500 – 4000`            | Use high threshold to filter flicker, reflections           |
| **Ignore motion entirely**                        | `1000.0`             | Any                      | Disables detection (used for testing or dry runs)           |

**Other parameters:**
- `MOG2_HISTORY`: `500` → good balance for scenes that change slowly.
- `MOG2_DETECT_SHADOWS`: `true` → enables gray mask for shadows (value 127 in `fg_mask`).
- Input resolution: Downscale to `960×540` (16:9) for fast processing and reliable detection.
