package io.disco.plugin;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.charset.StandardCharsets;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;

import io.disco.ffi.DiscoFfi;

public final class HelloCommand implements CommandExecutor {

    private static final int OUT_CAPACITY = 256;

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        String name = args.length > 0 ? args[0] : sender.getName();
        try (Arena arena = Arena.ofConfined()) {
            byte[] nameBytes = name.getBytes(StandardCharsets.UTF_8);
            MemorySegment nameBuf = arena.allocate(nameBytes.length);
            MemorySegment.copy(nameBytes, 0, nameBuf, ValueLayout.JAVA_BYTE, 0, nameBytes.length);
            MemorySegment outBuf = arena.allocate(OUT_CAPACITY);

            int written = DiscoFfi.disco_hello(nameBuf, nameBytes.length, outBuf, OUT_CAPACITY);

            byte[] outBytes = new byte[written];
            MemorySegment.copy(outBuf, ValueLayout.JAVA_BYTE, 0, outBytes, 0, written);
            sender.sendMessage(new String(outBytes, StandardCharsets.UTF_8));
        }
        return true;
    }
}
