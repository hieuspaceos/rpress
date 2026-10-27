<?php
	/*	
	*	Goodlayers Page Builder Options Creation
	*/
	
	if( !class_exists('gdlr_core_page_builder_options') ){
		class gdlr_core_page_builder_options{
			
			private $options;
			private $values;
			
			function __construct( $options = array(), $values = array() ){
				$this->options = $options;
				$this->values = $values;
			}
			
			function get_content(){
				$ret  = '<div class="gdlr-core-page-builder-options" >';
				$ret .= $this->get_options_head();
				
				$ret .= '<div class="gdlr-core-page-builder-options-content">';
				$ret .= $this->get_options_tab();
				$ret .= '</div>'; 
				
				$ret .= '<div class="gdlr-core-page-builder-options-end">';
				$ret .= '<div class="gdlr-core-page-builder-options-save" id="gdlr-core-page-builder-options-save" >';
				$ret .= '<i class="fa fa-save"></i>' . esc_html__('Save Options', 'goodlayers-core');
				$ret .= '</div>';
				$ret .= '</div>';
				$ret .= '</div>';
				
				return $ret;
			}
			
			function get_options_head(){
				
				$ret  = '<div class="gdlr-core-page-builder-options-head">';
				$ret .= '<div class="gdlr-core-page-builder-options-head-title">';
				$ret .= '<i class="fa fa-gears"></i>';
				$ret .= esc_html__('Item Options', 'goodlayers-core');
				$ret .= '</div>'; // gdlr-core-page-builder-options-head-title
				
				$id = empty($this->values['id'])? '': $this->values['id'];
				$class = empty($this->values['class'])? '': $this->values['class'];
				$ret .= '<div class="gdlr-core-page-builder-options-head-right" >';
				$ret .= '<span class="gdlr-core-page-builder-options-head-id-title" >' . esc_html__('Elem ID :', 'goodlayers-core') . '</span>';
				$ret .= '<input type="text" class="gdlr-core-page-builder-options-head-id" data-type="text" data-slug="id" value="' . esc_attr($id) . '" />';
				$ret .= '<span class="gdlr-core-page-builder-options-head-id-title" >' . esc_html__('Elem Class :', 'goodlayers-core') . '</span>';
				$ret .= '<input type="text" class="gdlr-core-page-builder-options-head-class" data-type="text" data-slug="class" value="' . esc_attr($class) . '" />';
				$ret .= '<span class="gdlr-core-page-builder-options-head-close" id="gdlr-core-page-builder-options-head-close" ></span>';
				$ret .= '</div>'; 
				$ret .= '</div>'; // gdlr-core-page-builder-options-head
				
				return $ret;
			}
			
			function get_options_tab(){
				
				// tab head
				$active = true;
				$ret  = '<div class="gdlr-core-page-builder-options-tab-head" id="gdlr-core-page-builder-options-tab-head" >';
				foreach( $this->options as $tab_slug => $tab_options ){
					$ret .= '<div class="gdlr-core-page-builder-options-tab-head-item ' . ($active? 'gdlr-core-active': '') . '" data-tab-slug="' . esc_attr($tab_slug) . '" >';
					$ret .= gdlr_core_escape_content($tab_options['title']);
					$ret .= '</div>';
					
					$active = false;
				}
				$ret .= '</div>';
				
				// tab content
				$active = true;
				$ret .= '<div class="gdlr-core-page-builder-options-tab-content" id="gdlr-core-page-builder-options-tab-content" >';
				foreach( $this->options as $tab_slug => $tab_options ){
					$ret .= '<div class="gdlr-core-page-builder-option-tab-content-item ' . ($active? 'gdlr-core-active': '') . '" data-tab-slug="' . esc_attr($tab_slug) . '" >';
					foreach( $tab_options['options'] as $option_slug => $option ){
						$option['slug'] = $option_slug;
						if( isset($this->values[$option_slug]) ){
							$option['value'] = $this->values[$option_slug];
						}else if( !empty($this->values) && !empty($option['type']) && $option['type'] == 'custom' ){
							$option['value'] = '';
						}
						
						$ret .= gdlr_core_html_option::get_element($option);
					}
					$ret .= '</div>';
					
					$active = false;
				}
				$ret .= '</div>';
				
				return $ret;
			}
			

		} // gdlr_core_page_builder_options
	} // class_exists	