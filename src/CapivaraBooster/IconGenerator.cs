using System.Drawing;
using System.Drawing.Imaging;

namespace CapivaraBooster
{
    /// <summary>
    /// Generates a multi-resolution .ico file from a source image.
    /// </summary>
    public static class IconGenerator
    {
        /// <summary>
        /// Generates a multi-resolution .ico file from a PNG source.
        /// Creates icons at 16x16, 32x32, 48x48, 128x128, and 256x256 resolutions.
        /// </summary>
        /// <param name="sourcePath">Path to the source PNG image (logo.png)</param>
        /// <param name="outputPath">Path where the .ico file will be saved (app.ico)</param>
        public static void GenerateIcon(string sourcePath, string outputPath)
        {
            // Load the source image
            using Image sourceImage = Image.FromFile(sourcePath);

            // Define the target resolutions for a Windows .ico file
            int[] resolutions = { 16, 32, 48, 128, 256 };

            // Create a multi-resolution icon
            using (var icon = new Icon(resolutions.Length))
            {
                for (int i = 0; i < resolutions.Length; i++)
                {
                    int resolution = resolutions[i];
                    // Resize the image to the target resolution, maintaining aspect ratio with padding
                    using var resized = new Bitmap(resolution, resolution);
                    using var graphics = Graphics.FromImage(resized);
                    graphics.CompositingQuality = System.Drawing.Drawing2D.CompositingQuality.HighQuality;
                    graphics.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.HighQuality;
                    graphics.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
                    
                    // Fill with transparent background
                    graphics.Clear(Color.Transparent);
                    
                    // Calculate centered placement
                    int x = (resolution - sourceImage.Width) / 2;
                    int y = (resolution - sourceImage.Height) / 2;
                    
                    // Draw the source image centered on the transparent background
                    graphics.DrawImage(sourceImage, new Rectangle(x, y, sourceImage.Width, sourceImage.Height));
                    
                    // Add the resized frame to the icon
                    icon.Frames[i] = icon.GetFrame(FrameDimension.FrameDimensionType) ?? 
                                     IconFrame.FromBitmap(resized);
                }

                // Save the .ico file
                icon.Save(outputPath);
            }

            Console.WriteLine($"✅ Multi-resolution .ico generated: {outputPath}");
        }
    }
}