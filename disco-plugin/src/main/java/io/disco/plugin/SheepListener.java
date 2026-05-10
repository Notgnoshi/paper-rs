package io.disco.plugin;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.ByteBuffer;
import java.util.UUID;

import org.bukkit.DyeColor;
import org.bukkit.entity.Sheep;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerInteractEntityEvent;

import io.disco.ffi.DiscoFfi;

public final class SheepListener implements Listener {

    private static final DyeColor[] COLORS = DyeColor.values();

    @EventHandler
    public void onInteract(PlayerInteractEntityEvent event) {
        if (!(event.getRightClicked() instanceof Sheep sheep)) {
            return;
        }
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment uuidBuf = arena.allocate(16);
            MemorySegment.copy(uuidToBytes(sheep.getUniqueId()), 0, uuidBuf, ValueLayout.JAVA_BYTE, 0, 16);
            int colorIdx = DiscoFfi.disco_pick_sheep_color(uuidBuf, 16);
            sheep.setColor(COLORS[colorIdx]);
        }
    }

    private static byte[] uuidToBytes(UUID uuid) {
        ByteBuffer buf = ByteBuffer.allocate(16);
        buf.putLong(uuid.getMostSignificantBits());
        buf.putLong(uuid.getLeastSignificantBits());
        return buf.array();
    }
}
