const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("SentientOS Zig Runtime Container v0.1.0\n", .{});
    try stdout.print("TSO Mode Active\n", .{});
}
