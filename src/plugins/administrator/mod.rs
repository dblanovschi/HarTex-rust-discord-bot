crate mod clean;
crate mod invites;
crate mod lockdown;
crate mod nickname_manipulation;
crate mod roles;
crate mod slowmode;
crate mod voice_manipulation;

mod webconfig_list;

crate use webconfig_list::WebconfigListCommand;
