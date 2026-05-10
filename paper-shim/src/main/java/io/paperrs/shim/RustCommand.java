package io.paperrs.shim;

import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;

/**
 * Generic Bukkit Command subclass that forwards execution and tab completion
 * to a Rust handler keyed by handlerId. Registered programmatically via
 * Server.getCommandMap() so plugin.yml never needs to declare commands added
 * by Rust code.
 */
public final class RustCommand extends Command {

    private final long handlerId;

    public RustCommand(String name, long handlerId) {
        super(name);
        this.handlerId = handlerId;
    }

    @Override
    public boolean execute(CommandSender sender, String label, String[] args) {
        return PaperRs.dispatchCommand(handlerId, sender, args);
    }
}
