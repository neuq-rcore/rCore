using System.IO.Ports;

// VisionFive2 uses 115200 as its default baud rate
const int BAUD = 115200;

if (args.Length < 2)
{
    Console.WriteLine("Usage: dotnet run <portName> <ELF-file>");
    return;
}

string portName = args[0];
string elf = args[1];

var serialPort = new SerialPort(portName, BAUD);

Console.WriteLine($"[!] Trying to open the port at: {portName}");

try
{
    serialPort.Open();
}
catch (Exception e)
{
    Console.WriteLine($"[ERR] Error occurred when opening the port. {e.Message}");
    return;
}

Console.CancelKeyPress += (_, _) =>
{
    try
    {
        lock (serialPort)
        {
            serialPort.Close();
        }
    }
    catch (Exception)
    {
        // Ignore
    }
};

Console.WriteLine("[!] Press Ctrl+C to exit");

serialPort.DataReceived += SerialPort_DataReceived;

while (true)
{
    var keyPressed = Console.ReadKey(true);

    if (keyPressed.Key == ConsoleKey.X && (keyPressed.Modifiers & ConsoleModifiers.Control) == ConsoleModifiers.Control)
    {
        Console.WriteLine(@"[VF2] Writing elf bytes to the serial port.");
        SerialPort_WriteELF(serialPort, elf);
        Console.WriteLine(@"[VF2] Write finished.");
    } else
    {
        serialPort.Write(keyPressed.KeyChar.ToString());
    }
}

void SerialPort_WriteELF(SerialPort serialPort, string filename)
{
    if (serialPort == null || !serialPort.IsOpen)
        throw new InvalidOperationException("[Error] Serial port is closed!");

    if (!File.Exists(filename))
        throw new FileNotFoundException(filename);
    
    lock (serialPort)
    {
        var data = File.ReadAllBytes(filename);

        var bytesToWrite = serialPort.BytesToWrite;

        for (int i = 0; i < data.Length; i += bytesToWrite)
        {
            serialPort.Write(data, i, bytesToWrite);
        }

        int remain = data.Length % bytesToWrite;
        if (remain != 0)
        {
            int offset = (data.Length / bytesToWrite) * bytesToWrite;
            serialPort.Write(data, offset, remain);
        }
    }
}

void SerialPort_DataReceived(object sender, SerialDataReceivedEventArgs e)
{
    var sp = (SerialPort)sender;
    Console.Write(sp.ReadExisting());
}