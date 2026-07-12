import { invoke } from "@tauri-apps/api/core";

const CHART_WIDTH = 520;
const CHART_HEIGHT = 220;
const PNG_SCALE = 2;

function inlineSvgStyles(source: SVGSVGElement): string {
  const clone = source.cloneNode(true) as SVGSVGElement;
  clone.setAttribute("xmlns", "http://www.w3.org/2000/svg");

  const sourceNodes = [source, ...Array.from(source.querySelectorAll("*"))];
  const cloneNodes = [clone, ...Array.from(clone.querySelectorAll("*"))];

  for (let index = 0; index < sourceNodes.length; index += 1) {
    const sourceNode = sourceNodes[index];
    const cloneNode = cloneNodes[index];
    if (!(cloneNode instanceof SVGElement)) {
      continue;
    }

    const computed = getComputedStyle(sourceNode);
    const fill = computed.fill;
    const stroke = computed.stroke;
    const strokeWidth = computed.strokeWidth;

    if (fill === "none" || fill === "rgba(0, 0, 0, 0)") {
      // Inline explicit "none" — standalone SVG defaults to black fill, which
      // turns polylines into solid filled polygons when CSS is not present.
      cloneNode.setAttribute("fill", "none");
    } else if (fill) {
      cloneNode.setAttribute("fill", fill);
    }
    if (stroke === "none") {
      cloneNode.setAttribute("stroke", "none");
    } else if (stroke) {
      cloneNode.setAttribute("stroke", stroke);
    }
    if (strokeWidth && strokeWidth !== "0px") {
      cloneNode.setAttribute("stroke-width", strokeWidth);
    }
    if (cloneNode instanceof SVGTextElement) {
      cloneNode.setAttribute("font-size", computed.fontSize);
      cloneNode.setAttribute("font-family", computed.fontFamily);
    }
  }

  source.querySelectorAll("stop").forEach((stop, index) => {
    const cloneStop = clone.querySelectorAll("stop")[index];
    if (cloneStop) {
      cloneStop.setAttribute("stop-color", getComputedStyle(stop).stopColor);
    }
  });

  return new XMLSerializer().serializeToString(clone);
}

async function renderPngBytes(svg: SVGSVGElement): Promise<Uint8Array> {
  const svgString = inlineSvgStyles(svg);
  const blob = new Blob([svgString], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);

  try {
    return await new Promise<Uint8Array>((resolve, reject) => {
      const image = new Image();
      image.onload = () => {
        const canvas = document.createElement("canvas");
        canvas.width = CHART_WIDTH * PNG_SCALE;
        canvas.height = CHART_HEIGHT * PNG_SCALE;
        const context = canvas.getContext("2d");
        if (!context) {
          reject(new Error("Canvas is unavailable in this environment."));
          return;
        }

        context.fillStyle = "#ffffff";
        context.fillRect(0, 0, canvas.width, canvas.height);
        context.drawImage(image, 0, 0, canvas.width, canvas.height);
        canvas.toBlob((pngBlob) => {
          if (!pngBlob) {
            reject(new Error("Failed to encode the chart as PNG."));
            return;
          }
          void pngBlob.arrayBuffer().then((buffer) => resolve(new Uint8Array(buffer)));
        }, "image/png");
      };
      image.onerror = () => reject(new Error("Failed to render the chart image."));
      image.src = url;
    });
  } finally {
    URL.revokeObjectURL(url);
  }
}

export async function exportChartAsSvg(
  svg: SVGSVGElement,
  defaultStem: string,
): Promise<boolean> {
  const fileName = await invoke<string | null>("save_export_file", {
    exportType: "svg",
    fileStem: defaultStem,
    contents: Array.from(new TextEncoder().encode(inlineSvgStyles(svg))),
  });
  return fileName !== null;
}

export async function exportChartAsPng(
  svg: SVGSVGElement,
  defaultStem: string,
): Promise<boolean> {
  const fileName = await invoke<string | null>("save_export_file", {
    exportType: "png",
    fileStem: defaultStem,
    contents: Array.from(await renderPngBytes(svg)),
  });
  return fileName !== null;
}
