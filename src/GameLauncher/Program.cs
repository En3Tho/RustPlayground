// See https://aka.ms/new-console-template for more information

using System.Diagnostics;

var processStartInfo = new ProcessStartInfo()
{
    FileName = args[1],
    Arguments = string.Join(" ", args[1..])
};

var process = Process.Start(processStartInfo);

if (process is null)
{
    throw new("Unable to start process");
}

var path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "current.game");

File.WriteAllLines(path, new[]
{
    process.Id.ToString(),
    process.MainModule!.FileName!
});

await process.WaitForExitAsync();