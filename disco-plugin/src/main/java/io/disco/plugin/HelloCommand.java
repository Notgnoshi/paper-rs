package io.disco.plugin;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;

public final class HelloCommand implements CommandExecutor {

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        String name = args.length > 0 ? args[0] : sender.getName();
        sender.sendMessage(hello(name));
        return true;
    }

    private static native String hello(String name);
}
