<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('columnize', 'gdlr_core_pb_element_columnize'); 
	
	if( !class_exists('gdlr_core_pb_element_columnize') ){
		class gdlr_core_pb_element_columnize{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-columns',
					'title' => esc_html__('Columnize', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'column-number' => array(
								'title' => esc_html__('Column Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 3
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Columnize item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),						
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(		
							'font-size' => array(
								'title' => esc_html__('Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'description' => esc_html__('Leaving this field blank will display the default font size from theme options', 'goodlayers-core'),
							),	
							'border-size' => array(
								'title' => esc_html__('Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '1px'
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#e0e0e0'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'column-gap' => array(
								'title' => esc_html__('Column Gap', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '60px'
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
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-columnize-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-columnize-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'content' => esc_html__('Columnize item sample content', 'goodlayers-core'),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default size
				$settings['column-number'] = (empty($settings['column-number']) || $settings['column-number'] == 3)? '': $settings['column-number'];
				$settings['column-gap'] = (empty($settings['column-gap']) || $settings['column-gap'] == '30px')? '': $settings['column-gap'];
				$settings['border-size'] = (empty($settings['border-size']) || $settings['border-size'] == '1px')? '': $settings['border-size'];
				$settings['border-color'] = (empty($settings['border-color']) || $settings['border-color'] == '#e0e0e0')? '': $settings['border-color'];
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-columnize-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-columnize-item-content gdlr-core-skin-divider" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['font-size'])? '': $settings['font-size'],
						'column-count' => $settings['column-number'],
						'-moz-column-count' => $settings['column-number'],
						'-webkit-column-count' => $settings['column-number'],
						'column-gap' => $settings['column-gap'],
						'-moz-column-gap' => $settings['column-gap'],
						'-webkit-column-gap' => $settings['column-gap'],
						'column-rule-width' => $settings['border-size'],
						'-moz-column-rule-width' => $settings['border-size'],
						'-webkit-column-rule-width' => $settings['border-size'],
						'column-rule-color' => $settings['border-color'],
						'-moz-column-rule-color' => $settings['border-color'],
						'-webkit-column-rule-color' => $settings['border-color'],
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	