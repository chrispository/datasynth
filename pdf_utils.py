import logging
import os
from fpdf import FPDF

def init_pdf(base_dir=None):
    """
    Initialize an FPDF object with DejaVu fonts registered for Unicode support.
    
    Args:
        base_dir: Optional base directory to look for fonts folder. 
                  If None, tries to determine from this file's location.
    
    Returns:
        An initialized FPDF instance.
    """
    if base_dir is None:
        base_dir = os.path.dirname(os.path.abspath(__file__))
        
    font_path = os.path.join(base_dir, "fonts", "DejaVuSans.ttf")
    
    pdf = FPDF()
    
    if os.path.exists(font_path):
        # Register variants
        # Note: uni=True is deprecated in newer fpdf2 versions but kept for compatibility 
        # with what might be installed. If it warns, we can remove it.
        # Check fpdf version if possible or try/except, but sticking to existing pattern for now.
        try:
            pdf.add_font("DejaVu", "", font_path, uni=True)
            pdf.add_font("DejaVu", "B", font_path, uni=True)
            pdf.add_font("DejaVu", "I", font_path, uni=True)
            pdf.add_font("DejaVu", "BI", font_path, uni=True)
        except TypeError:
             # Fallback for newer fpdf2 that removed 'uni' arg
            pdf.add_font("DejaVu", "", font_path)
            pdf.add_font("DejaVu", "B", font_path)
            pdf.add_font("DejaVu", "I", font_path)
            pdf.add_font("DejaVu", "BI", font_path)
    else:
        logging.warning(f"Font not found at {font_path}, falling back to standard fonts.")
        
    # Set default settings
    pdf.set_margins(10, 10, 10)
    pdf.set_auto_page_break(auto=True, margin=15)
    
    return pdf
