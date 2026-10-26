use crate::geom::Rect;

#[derive(Debug, Clone, Copy)]
pub struct MenuMetrics {
    pub height: i32,
    pub gap: i32,
    pub min_width: u32,
    pub max_width: u32,
}

pub fn place_menu(selection: Rect, screen: (u32, u32), menu: MenuMetrics) -> Rect {
    let (sw, sh) = (screen.0.max(1) as i32, screen.1.max(1) as i32);
    let width = selection
        .w
        .clamp(menu.min_width, menu.max_width)
        .min(sw as u32);
    let x = (selection.centre().0 - width as i32 / 2).clamp(0, (sw - width as i32).max(0));

    let below = selection.bottom() + menu.gap;
    let y = if below + menu.height <= sh {
        below
    } else {
        (selection.bottom() - menu.height).clamp(0, (sh - menu.height).max(0))
    };
    Rect::new(x, y, width, menu.height as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (u32, u32) = (3840, 2160);
    const MENU: MenuMetrics = MenuMetrics {
        height: 69,
        gap: 15,
        min_width: 390,
        max_width: 1080,
    };

    #[test]
    fn the_menu_sits_centred_below_the_selection() {
        let sel = Rect::new(900, 400, 1000, 500);
        let bar = place_menu(sel, SCREEN, MENU);
        assert_eq!(bar.y, sel.bottom() + MENU.gap);
        assert_eq!(bar.centre().0, sel.centre().0);
    }

    #[test]
    fn without_room_below_the_menu_overlaps_the_bottom_edge() {
        let bar = place_menu(Rect::new(0, 0, 3840, 2160), SCREEN, MENU);
        assert_eq!(bar.bottom(), SCREEN.1 as i32);
    }

    #[test]
    fn the_menu_is_clamped_onto_the_screen() {
        let right = place_menu(Rect::new(3740, 900, 100, 100), SCREEN, MENU);
        assert!(right.right() <= SCREEN.0 as i32);
        let tall = MenuMetrics {
            height: 5000,
            ..MENU
        };
        assert_eq!(place_menu(Rect::new(0, 0, 800, 600), SCREEN, tall).y, 0);
    }
}
