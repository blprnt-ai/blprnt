import os
app_path = os.environ.get('APP_PATH')

files = [app_path]
symlinks = {'Applications': '/Applications'}

badge_icon = None
icon_locations = {
    'blprnt.app': (140, 120),
    'Applications': (500, 120)
}

background = None
show_status_bar = False
show_tab_view = False
show_toolbar = False
show_pathbar = False
show_sidebar = False
sidebar_width = 180

window_rect = ((100, 100), (640, 360))
default_view = 'icon-view'

show_icon_preview = False

arrange_by = None
grid_offset = (0, 0)
grid_spacing = 100
scroll_position = (0, 0)
label_pos = 'bottom'
text_size = 16
icon_size = 128