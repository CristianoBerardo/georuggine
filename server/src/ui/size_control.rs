use ratatui::layout::Rect;

pub(crate) const MIN_WIDTH: u16 = 80;
pub(crate) const MIN_HEIGHT: u16 = 29;

pub(crate) fn size_too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: u16, height: u16) -> Rect {
        Rect::new(0, 0, width, height)
    }

    #[test]
    fn dimensione_esattamente_alla_soglia_va_bene() {
        assert!(!size_too_small(area(MIN_WIDTH, MIN_HEIGHT)));
    }

    #[test]
    fn dimensione_abbondante_va_bene() {
        assert!(!size_too_small(area(MIN_WIDTH + 20, MIN_HEIGHT + 10)));
    }

    #[test]
    fn larghezza_sotto_soglia_e_troppo_piccola() {
        assert!(size_too_small(area(MIN_WIDTH - 1, MIN_HEIGHT)));
    }

    #[test]
    fn altezza_sotto_soglia_e_troppo_piccola() {
        assert!(size_too_small(area(MIN_WIDTH, MIN_HEIGHT - 1)));
    }

    #[test]
    fn area_nulla_e_troppo_piccola() {
        assert!(size_too_small(area(0, 0)));
    }
}
