<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('breadcrumbs', 'gdlr_core_pb_element_breadcrumbs'); 
	
	if( !class_exists('gdlr_core_pb_element_breadcrumbs') ){
		class gdlr_core_pb_element_breadcrumbs{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-address-book',
					'title' => esc_html__('Breadcrumbs', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);			
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class = empty($settings['class'])? '': $settings['class'];
				$extra_class = empty($settings['text-align'])? '': ' gdlr-core-' . $settings['text-align'] . '-align';
				$ret  = '<div class="gdlr-core-breadcrumbs-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// display
				if( !function_exists('bcn_display') ){
					$message = wp_kses(__('Please install and activate the "<a target="_blank" href="https://wordpress.org/plugins/breadcrumb-navxt/" >Breadcrumbs NavXT</a>" plugin to show breadcrumbs.', 'goodlayers-core'), 
						array( 'a' => array('target'=>array(), 'href'=>array()) ));
				}else{
					ob_start();
					bcn_display();
					$ret .= ob_get_contents();
					ob_end_clean();
				}
				
				if( !empty($message) ){
					$ret .= '<div class="gdlr-core-external-plugin-message">' . gdlr_core_escape_content($message) . '</div>';
				}
				
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_breadcrumbs
	} // class_exists	