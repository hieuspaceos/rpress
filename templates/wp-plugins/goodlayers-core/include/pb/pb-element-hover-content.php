<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('hover-content', 'gdlr_core_pb_element_hover_content'); 
	
	if( !class_exists('gdlr_core_pb_element_hover_content') ){
		class gdlr_core_pb_element_hover_content{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-inbox',
					'title' => esc_html__('Hover Content', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'content' => array(
						'title' => esc_html__('Content', 'goodlayers-core'),
						'options' => array(
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text'
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text'
							),
							'button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#',
							),
							'button-link-target' => array(
								'title' => esc_html__('Button Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'align' => array(
								'title' => esc_html__('Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-font-size' => array(
								'title' => esc_html__('Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'title-bottom-margin' => array(
								'title' => esc_html__('Title Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-font-size' => array(
								'title' => esc_html__('Content Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-bottom-margin' => array(
								'title' => esc_html__('Content Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'button-background-color' => array(
								'title' => esc_html__('Button Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),	
					'hover-content' => array(
						'title' => esc_html__('Hover Content', 'goodlayers-core'),
						'options' => array(
							'b-title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
							),
							'b-content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'b-button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text'
							),
							'b-button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#',
							),
							'b-button-link-target' => array(
								'title' => esc_html__('Button Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
							),
						),
					),		
					'hover-style' => array(
						'title' => esc_html__('Hover Style', 'goodlayers-core'),
						'options' => array(
							'b-align' => array(
								'title' => esc_html__('Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'b-title-color' => array(
								'title' => esc_html__('Title Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-title-font-size' => array(
								'title' => esc_html__('Title Hover Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'b-title-bottom-margin' => array(
								'title' => esc_html__('Title Hover Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'b-content-color' => array(
								'title' => esc_html__('Content Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-content-font-size' => array(
								'title' => esc_html__('Content Hover Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'b-content-bottom-margin' => array(
								'title' => esc_html__('Content Hover Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'b-button-background-color' => array(
								'title' => esc_html__('Button Background Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'item-padding' => array(
								'title' => esc_html__('Item Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
							),
							'padding-bottom' => array(
								'title' => esc_html__('Margin Bottom ( Wrapper )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					),
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-hover-content-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var hover_content_elem = jQuery('#gdlr-core-preview-hover-content-<?php echo esc_attr($id); ?>').parent();
	new gdlr_core_sync_height(hover_content_elem.closest('.gdlr-core-page-builder-body'));
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;

				// default variable
				if( empty($settings) ){
					$settings = array(
						'padding-bottom' => $gdlr_core_item_pdb,
						'align' => 'center',
						'title' => 'Sameple Title',
						'content' => 'Sample Content',
						'b-align' => 'center',
						'b-title' => 'Sameple Title Back',
						'b-content' => 'Sample Content Back'
					);
				}
				
				// generate uniq id
				if( empty($settings['id']) ){
					global $gdlr_core_hover_content_id;
					$gdlr_core_hover_content_id = empty($gdlr_core_hover_content_id)? array(): $gdlr_core_hover_content_id;

					// generate unique id so it does not get overwritten in admin area
					$rnd_hover_content_id = mt_rand(0, 99999);
					while( in_array($rnd_hover_content_id, $gdlr_core_hover_content_id) ){
						$rnd_hover_content_id = mt_rand(0, 99999);
					}
					$gdlr_core_hover_content_id[] = $rnd_hover_content_id;
					$sync_height_id = 'gdlr-core-hover-content-' . $rnd_hover_content_id;
				}else{
					$sync_height_id = $settings['id'];
				}
				
				// start printing item
				$ret  = '<div class="gdlr-core-hover-content-item gdlr-core-item-mgb clearfix gdlr-core-' . (empty($settings['align'])? 'center': $settings['align']) . '-align" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('margin-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// front
				$ret .= '<div class="gdlr-core-hover-content-front gdlr-core-js" data-sync-height="' . esc_attr($sync_height_id) . '" data-sync-height-center ' . gdlr_core_esc_style(array(
					'padding' => empty($settings['item-padding'])? '': $settings['item-padding']
				)) .' >';
				$ret .= '<div class="gdlr-core-hover-content" >';
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-hover-content-title" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['title-color'])? '': $settings['title-color'],
						'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
						'margin-bottom' => empty($settings['title-bottom-margin'])? '': $settings['title-bottom-margin']
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-hover-content-content" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['content-color'])? '': $settings['content-color'],
						'font-size' => empty($settings['content-font-size'])? '': $settings['content-font-size'],
						'margin-bottom' => empty($settings['content-bottom-margin'])? '': $settings['content-bottom-margin']
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				if( !empty($settings['button-text']) && !empty($settings['button-link']) ){
					$ret .= '<div class="gdlr-core-hover-content-button" >';
					$ret .= '<a class="gdlr-core-button" href="' . esc_attr($settings['button-link']) . '" ';
					$ret .= empty($settings['button-link-target'])? '': ' target="' . esc_attr($settings['button-link-target']) . '" ';
					$ret .= '>' . gdlr_core_text_filter($settings['button-text']) . '</a>';
					$ret .= gdlr_core_esc_style(array(
						'background-color' => empty($settings['button-background-color'])? '': $settings['button-background-color']
					));
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-hover-content
				$ret .= '</div>'; // gdlr-core-hover-content-front

				// back
				$ret .= '<div class="gdlr-core-hover-content-back gdlr-core-js" data-sync-height="' . esc_attr($sync_height_id) . '" data-sync-height-center ' . gdlr_core_esc_style(array(
					'padding' => empty($settings['item-padding'])? '': $settings['item-padding']
				)) .' >';
				$ret .= '<div class="gdlr-core-hover-content" >';
				if( !empty($settings['b-title']) ){
					$ret .= '<h3 class="gdlr-core-hover-content-title" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['b-title-color'])? '': $settings['b-title-color'],
						'font-size' => empty($settings['b-title-font-size'])? '': $settings['b-title-font-size'],
						'margin-bottom' => empty($settings['b-title-bottom-margin'])? '': $settings['b-title-bottom-margin']
					)) . ' >' . gdlr_core_text_filter($settings['b-title']) . '</h3>';
				}
				if( !empty($settings['b-content']) ){
					$ret .= '<div class="gdlr-core-hover-content-content" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['b-content-color'])? '': $settings['b-content-color'],
						'font-size' => empty($settings['b-content-font-size'])? '': $settings['b-content-font-size'],
						'margin-bottom' => empty($settings['b-content-bottom-margin'])? '': $settings['b-content-bottom-margin']
					)) . ' >' . gdlr_core_content_filter($settings['b-content']) . '</div>';
				}
				if( !empty($settings['b-button-text']) && !empty($settings['b-button-link']) ){
					$ret .= '<div class="gdlr-core-hover-content-button" >';
					$ret .= '<a class="gdlr-core-button" href="' . esc_attr($settings['b-button-link']) . '" ';
					$ret .= empty($settings['b-button-link-target'])? '': ' target="' . esc_attr($settings['b-button-link-target']) . '" ';
					$ret .= gdlr_core_esc_style(array(
						'background-color' => empty($settings['b-button-background-color'])? '': $settings['b-button-background-color']
					));
					$ret .= '>' . gdlr_core_text_filter($settings['b-button-text']) . '</a>';
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-hover-content
				$ret .= '</div>'; // gdlr-core-hover-content-back

				$ret .= '</div>'; // gdlr-core-hover-content-item
				
				return $ret;
			}		
			
		} // gdlr_core_pb_element_hover_content
	} // class_exists	